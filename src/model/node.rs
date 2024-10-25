use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Clone, Serialize)]
pub struct Node {
    #[serde(rename = "id")]
    pub head_id: Option<String>,
    #[serde(rename = "total")]
    pub stats: NodeStats,
    pub stats_file: Option<String>,
    pub region: String,
    pub max_players: usize,
    pub persisted: bool,
    pub reserved: bool,
    pub online: Arc<AtomicBool>,
}

#[derive(Serialize)]
pub struct NodeSummary(pub Node);

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeStats {
    #[serde(default)]
    pub online_nodes: usize,
    #[serde(default)]
    pub offline_nodes: usize,
    pub total_games: u64,
    pub active_games: usize,
    pub transactions: u64,
    pub bytes: u64,

    pub kills: HashMap<String, u64>,
    pub total_kills: u64,
    pub kills_leaderboard: Vec<LeaderboardEntry>,
    pub items: HashMap<String, u64>,
    pub total_items: u64,
    pub items_leaderboard: Vec<LeaderboardEntry>,
    pub secrets: HashMap<String, u64>,
    pub total_secrets: u64,
    pub secrets_leaderboard: Vec<LeaderboardEntry>,

    pub player_play_time: HashMap<String, Vec<u128>>,
    pub total_play_time: u128,

    #[serde(skip)]
    pub pending_transactions: HashMap<Vec<u8>, StateUpdate>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct LeaderboardEntry(String, u64);

#[derive(Clone, Debug)]
pub struct StateUpdate {
    pub player: String,
    pub bytes: u64,
    pub kills: u64,
    pub items: u64,
    pub secrets: u64,
    pub time: Vec<u128>,
}
