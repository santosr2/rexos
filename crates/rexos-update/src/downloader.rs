//! Update download with resume support

use crate::{UpdateError, UpdateInfo};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download
    pub total: u64,
    /// Bytes downloaded so far
    pub downloaded: u64,
    /// Download speed in bytes per second
    pub speed: u64,
    /// Estimated time remaining in seconds
    pub eta: u64,
    /// Current state
    pub state: DownloadState,
}

impl DownloadProgress {
    /// Get progress as percentage (0-100)
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            0
        } else {
            ((self.downloaded as f64 / self.total as f64) * 100.0) as u8
        }
    }
}

/// Download state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Verifying,
}

/// Downloads updates with resume support
pub struct UpdateDownloader {
    download_dir: PathBuf,
    max_retries: u32,
    client: reqwest::Client,
    progress: Arc<Mutex<Option<DownloadProgress>>>,
}

impl UpdateDownloader {
    /// Create a new downloader
    pub fn new(download_dir: PathBuf, max_retries: u32) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .user_agent(format!("RexOS/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            download_dir,
            max_retries,
            client,
            progress: Arc::new(Mutex::new(None)),
        }
    }

    /// Download an update
    pub async fn download(&self, update: &UpdateInfo) -> Result<PathBuf, UpdateError> {
        // Ensure download directory exists
        fs::create_dir_all(&self.download_dir)?;

        // Determine output path
        let filename = format!("rexos-{}.tar.gz", update.version);
        let output_path = self.download_dir.join(&filename);
        let partial_path = self.download_dir.join(format!("{}.partial", filename));

        // Check for existing partial download
        let resume_from = if partial_path.exists() {
            fs::metadata(&partial_path)?.len()
        } else {
            0
        };

        tracing::info!(
            "Downloading {} ({} bytes, resuming from {})",
            update.download_url,
            update.size,
            resume_from
        );

        // Initialize progress
        {
            let mut progress = self.progress.lock().unwrap();
            *progress = Some(DownloadProgress {
                total: update.size,
                downloaded: resume_from,
                speed: 0,
                eta: 0,
                state: DownloadState::Downloading,
            });
        }

        // Attempt download with retries
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            if attempt > 0 {
                tracing::warn!("Retry attempt {} of {}", attempt + 1, self.max_retries);
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }

            match self
                .download_with_resume(&update.download_url, &partial_path, resume_from)
                .await
            {
                Ok(()) => {
                    // Rename partial to final
                    fs::rename(&partial_path, &output_path)?;

                    // Update progress
                    {
                        let mut progress = self.progress.lock().unwrap();
                        if let Some(ref mut p) = *progress {
                            p.state = DownloadState::Completed;
                            p.downloaded = update.size;
                        }
                    }

                    return Ok(output_path);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // Update progress to failed
        {
            let mut progress = self.progress.lock().unwrap();
            if let Some(ref mut p) = *progress {
                p.state = DownloadState::Failed;
            }
        }

        Err(last_error.unwrap_or_else(|| UpdateError::DownloadFailed("Unknown error".into())))
    }

    /// Download with resume support
    async fn download_with_resume(
        &self,
        url: &str,
        path: &PathBuf,
        resume_from: u64,
    ) -> Result<(), UpdateError> {
        let mut request = self.client.get(url);

        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request.send().await?;

        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            return Err(UpdateError::DownloadFailed(format!(
                "Server returned {}",
                response.status()
            )));
        }

        // Open file for appending
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        // Stream the response
        let mut stream = response.bytes_stream();
        let mut downloaded = resume_from;
        let mut last_update = std::time::Instant::now();
        let mut bytes_since_update = 0u64;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| UpdateError::DownloadFailed(e.to_string()))?;
            file.write_all(&chunk)?;

            downloaded += chunk.len() as u64;
            bytes_since_update += chunk.len() as u64;

            // Update progress every 100ms
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_update);

            if elapsed.as_millis() >= 100 {
                let speed = (bytes_since_update as f64 / elapsed.as_secs_f64()) as u64;

                let mut progress = self.progress.lock().unwrap();
                if let Some(ref mut p) = *progress {
                    p.downloaded = downloaded;
                    p.speed = speed;

                    if speed > 0 {
                        p.eta = (p.total - downloaded) / speed;
                    }
                }

                last_update = now;
                bytes_since_update = 0;
            }
        }

        file.sync_all()?;
        Ok(())
    }

    /// Get current progress
    pub fn progress(&self) -> Option<DownloadProgress> {
        self.progress.lock().unwrap().clone()
    }

    /// Cancel current download
    pub fn cancel(&self) {
        let mut progress = self.progress.lock().unwrap();
        if let Some(ref mut p) = *progress {
            p.state = DownloadState::Failed;
        }
    }

    /// Clean up partial downloads
    pub fn cleanup(&self) -> Result<(), UpdateError> {
        if self.download_dir.exists() {
            for entry in fs::read_dir(&self.download_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().is_some_and(|e| e == "partial") {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    /// Get available disk space
    pub fn available_space(&self) -> Result<u64, UpdateError> {
        // Use statvfs on Unix
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;

            let path = self.download_dir.as_os_str();
            let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };

            let c_path = std::ffi::CString::new(path.as_bytes()).map_err(|e| {
                UpdateError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
            })?;

            let result = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };

            if result == 0 {
                Ok(stat.f_bavail as u64 * stat.f_bsize as u64)
            } else {
                Err(UpdateError::Io(std::io::Error::last_os_error()))
            }
        }

        #[cfg(not(unix))]
        {
            // Fallback: assume enough space
            Ok(u64::MAX)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_percent() {
        let progress = DownloadProgress {
            total: 100,
            downloaded: 50,
            speed: 10,
            eta: 5,
            state: DownloadState::Downloading,
        };

        assert_eq!(progress.percent(), 50);
    }

    #[test]
    fn test_progress_percent_zero_total() {
        let progress = DownloadProgress {
            total: 0,
            downloaded: 0,
            speed: 0,
            eta: 0,
            state: DownloadState::Pending,
        };

        assert_eq!(progress.percent(), 0);
    }
}
