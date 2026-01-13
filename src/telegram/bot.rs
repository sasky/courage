use anyhow::{Context, Result};
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::net::Download;
use teloxide::types::{MediaKind, MessageKind};

use crate::config::Config;
use crate::llm::{ActionType, AnthropicClient, LlmResponse, ResponseStatus};
use crate::people::{
    read_all_people, generate_person_markdown, write_person_file, update_person_file,
    PersonFrontmatter, Relationships, PersonSections,
};
use crate::state::ConversationState;
use crate::transcription::WhisperClient;

/// The main bot handler
pub struct CourageBot {
    config: Config,
    whisper: WhisperClient,
    claude: AnthropicClient,
    state: ConversationState,
}

impl CourageBot {
    pub fn new(config: Config) -> Self {
        let whisper = WhisperClient::new(config.openai_api_key.clone());
        let claude = AnthropicClient::new(
            config.anthropic_api_key.clone(),
            config.claude_model.clone(),
        );
        let state = ConversationState::new();

        Self {
            config,
            whisper,
            claude,
            state,
        }
    }

    /// Run the bot
    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting Courage bot...");

        let bot = Bot::new(&self.config.telegram_token);
        let bot_arc = Arc::new(self);

        teloxide::repl(bot, move |bot_instance: Bot, msg: Message| {
            let handler = bot_arc.clone();
            async move {
                if let Err(e) = handler.handle_message(&bot_instance, &msg).await {
                    tracing::error!("Error handling message: {}", e);
                    let _ = bot_instance
                        .send_message(msg.chat.id, format!("Error: {}", e))
                        .await;
                }
                Ok(())
            }
        })
        .await;

        Ok(())
    }

    /// Handle an incoming message
    async fn handle_message(&self, bot: &Bot, msg: &Message) -> Result<()> {
        let chat_id = msg.chat.id;

        if let MessageKind::Common(common) = &msg.kind {
            match &common.media_kind {
                // Voice message
                MediaKind::Voice(voice) => {
                    self.handle_voice(bot, chat_id, &voice.voice.file.id).await?;
                }
                // Text message
                MediaKind::Text(text) => {
                    self.handle_text(bot, chat_id, &text.text).await?;
                }
                _ => {
                    bot.send_message(chat_id, "Please send a voice message or text.")
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Handle a voice message
    async fn handle_voice(&self, bot: &Bot, chat_id: ChatId, file_id: &str) -> Result<()> {
        // Let user know we're processing
        bot.send_message(chat_id, "Transcribing...").await?;

        // Download the voice file
        let file = bot.get_file(file_id).await?;
        let mut audio_data = Vec::new();

        bot.download_file(&file.path, &mut audio_data).await?;

        // Transcribe with Whisper
        let transcript = self
            .whisper
            .transcribe(audio_data, "voice.ogg")
            .await
            .context("Failed to transcribe audio")?;

        tracing::info!("Transcript: {}", transcript);

        // Process the transcript
        self.process_transcript(bot, chat_id, &transcript).await
    }

    /// Handle a text message
    async fn handle_text(&self, bot: &Bot, chat_id: ChatId, text: &str) -> Result<()> {
        // Check if this is a reply to a clarification question
        if let Some(pending) = self.state.take_pending(chat_id.0).await {
            // Continue the conversation with clarification
            return self
                .continue_with_clarification(bot, chat_id, &pending.transcript, text)
                .await;
        }

        // Otherwise, treat it as a new transcript
        self.process_transcript(bot, chat_id, text).await
    }

    /// Process a transcript (voice or text)
    async fn process_transcript(&self, bot: &Bot, chat_id: ChatId, transcript: &str) -> Result<()> {
        // Read existing people for context
        let people_context = read_all_people(&self.config.people_dir).await?;
        let people_list = people_context.to_prompt_list();

        // Send to Claude for processing
        bot.send_message(chat_id, "Processing...").await?;

        let response = self
            .claude
            .process_transcript(transcript, &people_list)
            .await
            .context("Failed to process with Claude")?;

        self.handle_llm_response(bot, chat_id, transcript, response)
            .await
    }

    /// Continue processing with user's clarification
    async fn continue_with_clarification(
        &self,
        bot: &Bot,
        chat_id: ChatId,
        original_transcript: &str,
        clarification: &str,
    ) -> Result<()> {
        let people_context = read_all_people(&self.config.people_dir).await?;
        let people_list = people_context.to_prompt_list();

        bot.send_message(chat_id, "Processing with clarification...").await?;

        let response = self
            .claude
            .continue_with_clarification(original_transcript, clarification, &people_list)
            .await
            .context("Failed to continue with clarification")?;

        self.handle_llm_response(bot, chat_id, original_transcript, response)
            .await
    }

    /// Handle the response from Claude
    async fn handle_llm_response(
        &self,
        bot: &Bot,
        chat_id: ChatId,
        transcript: &str,
        response: LlmResponse,
    ) -> Result<()> {
        match response.status {
            ResponseStatus::ClarificationNeeded => {
                // Store state and ask for clarification
                self.state
                    .set_pending(chat_id.0, transcript.to_string(), response.message.clone())
                    .await;

                let question = format!(
                    "❓ {}\n\n───────────────\n💡 Please reply with more detail",
                    response.message
                );
                bot.send_message(chat_id, question).await?;
            }

            ResponseStatus::Ready => {
                // Execute the actions
                let mut summaries = Vec::new();

                for action in &response.actions {
                    match action.action_type {
                        ActionType::Create => {
                            let frontmatter = PersonFrontmatter {
                                first_name: action.fields.get("first_name").cloned().unwrap_or_default(),
                                last_name: action.fields.get("last_name").cloned().unwrap_or_default(),
                                disambiguator: action.fields.get("disambiguator").cloned().unwrap_or_default(),
                                nickname: action.fields.get("nickname").cloned().unwrap_or_default(),
                                birthday: action.fields.get("birthday").cloned().unwrap_or_default(),
                                phone: action.fields.get("phone").cloned().unwrap_or_default(),
                                email: action.fields.get("email").cloned().unwrap_or_default(),
                                location: action.fields.get("location").cloned().unwrap_or_default(),
                                work: action.fields.get("work").cloned().unwrap_or_default(),
                                ..Default::default()
                            };

                            let relationships: Relationships = action.relationships.clone().into();
                            let sections: PersonSections = action.sections.clone().into();

                            let content = generate_person_markdown(&frontmatter, &relationships, &sections);
                            let filename = action.filename.trim_end_matches(".md");

                            write_person_file(&self.config.people_dir, filename, &content).await?;
                            summaries.push(format!("✨ Created {}", filename));
                        }

                        ActionType::Update => {
                            let relationships: Relationships = action.relationships.clone().into();
                            let sections: PersonSections = action.sections.clone().into();
                            let filename = action.filename.trim_end_matches(".md");

                            update_person_file(
                                &self.config.people_dir,
                                filename,
                                &action.fields,
                                &relationships,
                                &sections,
                            )
                            .await?;

                            summaries.push(format!("📝 Updated {}", filename));
                        }
                    }
                }

                if summaries.is_empty() {
                    bot.send_message(chat_id, "✅ No changes needed.").await?;
                } else {
                    let summary = format!(
                        "✅ Done!\n\n{}\n\n───────────────\n{}",
                        summaries.join("\n"),
                        response.message
                    );
                    bot.send_message(chat_id, summary).await?;
                }
            }

            ResponseStatus::Error => {
                bot.send_message(chat_id, format!("❌ Error: {}", response.message))
                    .await?;
            }
        }

        Ok(())
    }
}
