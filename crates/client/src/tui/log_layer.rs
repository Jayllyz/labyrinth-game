//! Forwards agent logs to the TUI.

use super::AgentEvent;
use std::sync::mpsc::Sender;
use tracing::{
    Subscriber,
    field::{Field, Visit},
    span,
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

/// Sends logs from agent threads to the TUI, in that agent's tab.
pub struct TuiLayer {
    events: Sender<AgentEvent>,
}

impl TuiLayer {
    pub const fn new(events: Sender<AgentEvent>) -> Self {
        Self { events }
    }
}

/// The agent name kept on the `agent` span, so events inside it know their tab.
struct AgentName(String);

impl<S> Layer<S> for TuiLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let name = FieldText::read("name", |visitor| attrs.record(visitor));
        if let (Some(name), Some(span)) = (name, ctx.span(id)) {
            span.extensions_mut().insert(AgentName(name));
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let text = FieldText::read("message", |visitor| event.record(visitor)).unwrap_or_default();
        let level = *event.metadata().level();
        let agent = ctx.event_scope(event).and_then(|scope| {
            scope
                .from_root()
                .find_map(|span| span.extensions().get::<AgentName>().map(|n| n.0.clone()))
        });

        match agent {
            // The TUI stops listening once the user quits it; the game plays on anyway.
            Some(agent) => drop(self.events.send(AgentEvent::Log { agent, level, text })),
            // Logs outside an agent have no tab, so they go to stderr as before.
            None => eprintln!("{level} {text}"),
        }
    }
}

/// Reads one field of an event or span as text.
struct FieldText {
    field: &'static str,
    text: Option<String>,
}

impl FieldText {
    /// The text of `field`, from whatever `record` feeds the visitor it gets.
    fn read(field: &'static str, record: impl FnOnce(&mut dyn Visit)) -> Option<String> {
        let mut visitor = Self { field, text: None };
        record(&mut visitor);
        visitor.text
    }
}

impl Visit for FieldText {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == self.field {
            self.text = Some(format!("{value:?}"));
        }
    }
}
