use std::path::PathBuf;

use tokio::fs::{File, OpenOptions};
use tokio::{fs, io};

#[derive(Default)]
pub struct FileStorage {
    pub root_path: String,
}

const TEMP_REQUEST_DIR: &str = "tmp/request/";
const LOCAL_REQUEST_DIR: &str = "local/request/";

impl FileStorage {

    pub fn new(root_path: String) -> Self {
        Self { root_path }
    }

    async fn ensure_dir(&self, path: &PathBuf) -> io::Result<()> {
        if !path.exists() {
            fs::create_dir_all(&path)
                .await?
        }

        return Ok(());
    }

    pub fn storage_path(&self) -> PathBuf {
        return PathBuf::from(self.root_path.as_str());
    }

    pub fn temp_req_path(&self) -> PathBuf {
        return PathBuf::new()
            .join(self.storage_path())
            .join(TEMP_REQUEST_DIR);
    }

    pub fn local_req_path(&self) -> PathBuf {
        return PathBuf::new()
            .join(self.storage_path())
            .join(LOCAL_REQUEST_DIR);
    }

    ////////////////
    ////////////////
    ////////////////

    pub async fn file_write(&self, path: &PathBuf) -> io::Result<File> {
        let destination = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(false)
            .open(path)
            .await?;

        return Ok(destination);
    }

    ////////////////
    ////////////////
    ////////////////

    pub async fn prepare_request(&self, request_id: u64) -> io::Result<PathBuf> {
        let temp = self.temp_req_path();

        self.ensure_dir(&temp)
            .await?;

        let tmp_req = temp.join(request_id.to_string().as_str());
        
        if tmp_req.exists() {
            fs::remove_file(&tmp_req)
                .await?;
        }

        return Ok(tmp_req);
    }

    pub async fn finalize_request(&self, request_id: u64) -> io::Result<PathBuf> {
        let data = request_id.to_string();

        let temp = self.temp_req_path();

        let tmp_req = temp.join(data.as_str());
        if !tmp_req.exists() || !tmp_req.is_file() {
            // TODO v2 check what we can do there
        }

        let local = self.local_req_path();
        self.ensure_dir(&local).await?;

        let local_req = local.join(data.as_str());
        if local_req.exists() {
            fs::remove_file(&local_req).await?;
        }

        fs::rename(tmp_req, &local_req).await?;

        return Ok(local_req);
    }

    pub async fn erase_request(&self, request_id: u64) -> io::Result<()> {
        let request = request_id.to_string();
        
        let temp = self.temp_req_path();
        let local = self.local_req_path();
        
        let tmp_req = temp.join(request.as_str());
        let local_req = local.join(request.as_str());

        if tmp_req.exists() {
            fs::remove_file(tmp_req).await?;
        }

        if local_req.exists() {
            fs::remove_file(local_req).await?;
        }

        return Ok(());
    }
}
