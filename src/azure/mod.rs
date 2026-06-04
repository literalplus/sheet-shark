use std::{collections::HashMap, time::Duration};

use arc_swap::ArcSwap;
use azure_core::http::{RequestContent, Url};
use azure_storage_blob::*;
use color_eyre::Result;
use futures::StreamExt;
use futures::channel::mpsc;
use futures_time::{stream::StreamExt as _, time::Duration as FuturesDuration};
use lazy_static::lazy_static;
use time::Date;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::config::{AzureConfig, Config};

lazy_static! {
    static ref UPLOAD_MANAGER: ArcSwap<UploadManager> = ArcSwap::from_pointee(UploadManager::new());
}

pub struct UploadManager {
    senders: Mutex<HashMap<Date, mpsc::UnboundedSender<String>>>,
}

impl UploadManager {
    fn new() -> Self {
        Self {
            senders: Mutex::new(HashMap::new()),
        }
    }
}

pub async fn enqueue_upload(day: Date, json_content: String) {
    let config = Config::get_quick();
    info!("Enqueueing upload for {}", day);

    if config.azure.is_none() {
        info!("Azure Blob Storage is disabled");
        return;
    }
    let azure_config = config.azure.clone().expect("Azure config not to disappear");

    let manager = UPLOAD_MANAGER.load();
    let mut senders = manager.senders.lock().await;

    senders.retain(|_, tx| !tx.is_closed());

    if let Some(tx) = senders.get(&day) {
        let _ = tx.unbounded_send(json_content);
        return;
    }

    let (tx, rx) = mpsc::unbounded::<String>();
    let _ = tx.unbounded_send(json_content);
    senders.insert(day, tx);
    drop(senders);

    let duration = FuturesDuration::from(Duration::from_secs(azure_config.debounce_seconds));

    tokio::spawn(async move {
        let mut debounced = rx.debounce(duration);
        while let Some(content) = StreamExt::next(&mut debounced).await {
            debug!("Debounced upload triggered for {}", day);
            match upload_to_azure(&azure_config, day, &content).await {
                Ok(_) => info!(day = %day, "Successfully uploaded to Azure Blob Storage"),
                Err(e) => error!(day = %day, error = %e, "Failed to upload to Azure Blob Storage"),
            }
        }
    });
}

async fn upload_to_azure(config: &AzureConfig, day: Date, json_content: &str) -> Result<()> {
    let blob_name = format!(
        "{:04}-{:02}/{:04}-{:02}-{:02}.json",
        day.year(),
        u8::from(day.month()),
        day.year(),
        u8::from(day.month()),
        day.day()
    );

    let url = Url::parse(&config.blob_sas_url.clone().unwrap())?;
    let service_client = BlobServiceClient::new(url, None, None)?;
    let blob_client = service_client.blob_client("shark-exports", &blob_name);
    let content = RequestContent::from(json_content.as_bytes().into());

    blob_client.upload(content, None).await?;

    Ok(())
}
