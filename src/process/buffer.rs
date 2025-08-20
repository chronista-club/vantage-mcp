use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 循環バッファでログを管理
#[derive(Debug, Clone)]
pub struct CircularBuffer {
    buffer: Arc<RwLock<VecDeque<String>>>,
    capacity: usize,
}

impl CircularBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// 新しい行を追加
    pub async fn push(&self, line: String) {
        let mut buffer = self.buffer.write().await;
        if buffer.len() >= self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(line);
    }

    /// 最新のN行を取得
    pub async fn get_last_n(&self, n: usize) -> Vec<String> {
        let buffer = self.buffer.read().await;
        buffer
            .iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .collect()
    }

    /// すべての行を取得
    pub async fn get_all(&self) -> Vec<String> {
        let buffer = self.buffer.read().await;
        buffer.iter().cloned().collect()
    }

    /// バッファをクリア
    pub async fn clear(&self) {
        let mut buffer = self.buffer.write().await;
        buffer.clear();
    }

    /// バッファのサイズを取得
    pub async fn len(&self) -> usize {
        let buffer = self.buffer.read().await;
        buffer.len()
    }
}