use std::io::SeekFrom;
use blake3::Hasher;
use tokio::fs::File;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

pub async fn blake3(file: &mut File) -> io::Result<String> {
    file.seek(SeekFrom::Start(0)).await?;

    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 8192]; // Read in chunks of 8KB

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize()
        .to_hex()
        .to_lowercase();

    return Ok(format!("0x{}", hash));
}

#[tokio::test]
async fn check_validity() {
    let mut file = File::open("app.apk").await.expect("");
    let hex = blake3(&mut file).await.expect("");
    println!("Result: {hex}");
    assert_eq!(hex, "cea56514b3de4832173b162947896760ea42a45b567773a3d1c0f5f05587e9ef");
}
