#[cfg(test)]
mod tests {
    use MemIgnite::store::Store;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn set_and_get_value() {
        let store = Store::new();

        store.set("a".into(), "1".into(), None).await;
        let val = store.get("a").await;

        assert_eq!(val, Some("1".to_string()));
    }

    #[tokio::test]
    async fn delete_key() {
        let store = Store::new();

        store.set("a".into(), "1".into(), None).await;
        let deleted = store.del("a").await;

        assert!(deleted);
        assert_eq!(store.get("a").await, None);
    }

    #[tokio::test]
    async fn ttl_expires_key() {
        let store = Store::new();

        store
            .set("a".into(), "1".into(), Some(Duration::from_secs(1)))
            .await;

        sleep(Duration::from_secs(2)).await;

        let val = store.get("a").await;
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn ttl_is_lazy() {
        let store = Store::new();

        store
            .set("a".into(), "1".into(), Some(Duration::from_secs(1)))
            .await;

        sleep(Duration::from_secs(2)).await;

        // First access triggers deletion
        assert_eq!(store.get("a").await, None);
    }

    #[tokio::test]
    async fn replay_skips_expired_key() {
        let store = Store::new();

        // expired timestamp in the past
        let past_ts = 1;

        store
            .apply_raw(&format!("SET a 1 EXAT {}", past_ts))
            .await;

        assert_eq!(store.get("a").await, None);
    }

    #[tokio::test]
    async fn replay_restores_valid_key() {
        let store = Store::new();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let future = now + 5;

        store
            .apply_raw(&format!("SET a 1 EXAT {}", future))
            .await;

        assert_eq!(store.get("a").await, Some("1".to_string()));
    }
}
