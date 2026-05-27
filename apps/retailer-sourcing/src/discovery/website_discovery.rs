use crate::app_event::RetailerSourcingEvent;
use crate::discovery::fetch::FetchCategoryPage;
use crate::discovery::{COMMAND_TYPE_PROCESS_MENU, Policy, PolicySubscriber};
use crate::source_retailer::io::RetailerHomepageRetrieved;
use kernel::io::{CommandError, CommandHandlerPort, DbPool};
use kernel::{ApplicationCommand, ApplicationEvent};
use retailer_guild::io::RetailerCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const COMMAND_TYPE_FETCH_NODE: &str = "FetchNode";
pub const EVENT_TYPE_MENU_PROCESSED: &str = "MenuProcessed";
pub const EVENT_TYPE_NODE_FETCHED: &str = "NodeFetched";

// ---- ProcessMenu (entry: the retrieved homepage) --------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMenu {
    pub retailer_code: RetailerCode,
    pub artifact_id: Uuid,
    pub url: String,
}

impl ProcessMenu {
    /// Intention: a retrieved homepage means one `ProcessMenu` for that page.
    pub fn from_event(event: &RetailerHomepageRetrieved) -> Self {
        ProcessMenu {
            retailer_code: event.retailer_code.clone(),
            artifact_id: event.artifact_id,
            url: event.url.clone(),
        }
    }
}

impl ApplicationCommand for ProcessMenu {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_PROCESS_MENU
    }
}

#[allow(dead_code)]
pub struct ProcessMenuHandler {
    pool: DbPool,
}

impl ProcessMenuHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<ProcessMenu, RetailerSourcingEvent> for ProcessMenuHandler {
    fn execute(&self, _cmd: ProcessMenu) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: parse the homepage menu into a category tree (see
        // docs/sourcing-flow.md). Should emit MenuProcessed with the tree nodes.
        Ok(Vec::new())
    }
}

// ---- FetchNode ------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchNode {
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl ApplicationCommand for FetchNode {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE_FETCH_NODE
    }
}

#[allow(dead_code)]
pub struct FetchNodeHandler {
    pool: DbPool,
}

impl FetchNodeHandler {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl CommandHandlerPort<FetchNode, RetailerSourcingEvent> for FetchNodeHandler {
    fn execute(&self, _cmd: FetchNode) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
        // TODO: begin walking a category-tree node's listing (see
        // docs/sourcing-flow.md). Should emit NodeFetched.
        Ok(Vec::new())
    }
}

// ---- events ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuProcessed {
    pub retailer_code: RetailerCode,
    pub node_urls: Vec<String>,
}

impl ApplicationEvent for MenuProcessed {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_MENU_PROCESSED
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeFetched {
    pub retailer_code: RetailerCode,
    pub url: String,
}

impl ApplicationEvent for NodeFetched {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_NODE_FETCHED
    }
}

// ---- policies -------------------------------------------------------------

/// A processed menu fans out to one FetchNode per category-tree node.
pub struct FetchNodePolicy;

impl Policy for FetchNodePolicy {
    type Command = FetchNode;
    const EVENT_NAME: &'static str = "MenuProcessed";
    const SOURCE: &'static str = "event:MenuProcessed";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<FetchNode> {
        // TODO: one FetchNode per node in the category tree
        Vec::new()
    }
}

pub type FetchNodeSubscriber = PolicySubscriber<FetchNodePolicy>;

/// A fetched node kicks off the category-page fetch (first page) for it.
pub struct NodeFetchCategoryPolicy;

impl Policy for NodeFetchCategoryPolicy {
    type Command = FetchCategoryPage;
    const EVENT_NAME: &'static str = "NodeFetched";
    const SOURCE: &'static str = "event:NodeFetched";

    fn commands(_event: &RetailerSourcingEvent) -> Vec<FetchCategoryPage> {
        // TODO: FetchCategoryPage (page 1) for the node's listing
        Vec::new()
    }
}

pub type NodeFetchCategorySubscriber = PolicySubscriber<NodeFetchCategoryPolicy>;
