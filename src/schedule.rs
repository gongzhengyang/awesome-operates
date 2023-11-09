use std::future::Future;
use std::time::Duration;

use tokio::time::MissedTickBehavior;

/// run with interval as second as unit
/// ```rust,no_run
/// use std::sync::Once;
///
/// use awesome_operates::schedule::run_with_interval;
///
/// static START: Once = Once::new();
///
/// async fn hello() {
///     println!("aaa");
/// }
///
/// START.call_once(|| {
///  #   run_with_interval(hello, 3);
/// });
/// ```
pub fn run_with_interval<F, Res>(mut f: F, interval: u64)
where
    F: 'static + FnMut() -> Res + Send,
    Res: 'static + Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        sleep_until_next_generate(interval).await;
        let mut interval_task = tokio::time::interval(Duration::from_secs(interval));
        interval_task.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            interval_task.tick().await;
            f().await;
        }
    });
}

/// don't change this logic
/// this will keep over at the begin of a second
async fn sleep_until_next_generate(interval: u64) {
    loop {
        if (chrono::Local::now().timestamp() as u64 % interval).eq(&0) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
