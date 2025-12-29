use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use nerve_search_adapter::client;
use crawler::search::SearchSchema;
use tempfile::tempdir;
use tantivy::{doc, Index};

struct CoreHarness {
    socket_path: PathBuf,
    shared_stream: Arc<Mutex<Option<UnixStream>>>,
    handle: thread::JoinHandle<std::io::Result<()>>,
}

impl CoreHarness {
    fn start(socket_path: PathBuf) -> Self {
        let stream_slot: Arc<Mutex<Option<UnixStream>>> = Arc::new(Mutex::new(None));
        let slot_clone = stream_slot.clone();
        let path_clone = socket_path.clone();

        let handle = thread::spawn(move || -> std::io::Result<()> {
            if path_clone.exists() {
                std::fs::remove_file(&path_clone)?;
            }
            let listener = UnixListener::bind(&path_clone)?;
            let (stream, _) = listener.accept()?;
            // keep a clone for controlled shutdown
            slot_clone
                .lock()
                .unwrap()
                .replace(stream.try_clone()?);
            nerve_core::connection::run(stream)
        });

        Self {
            socket_path,
            shared_stream: stream_slot,
            handle,
        }
    }

    fn wait_for_connection(&self, timeout: Duration) -> bool {
        let step = Duration::from_millis(20);
        let mut waited = Duration::from_millis(0);
        while waited <= timeout {
            if self.shared_stream.lock().unwrap().is_some() {
                return true;
            }
            thread::sleep(step);
            waited += step;
        }
        false
    }

    fn shutdown(&self) {
        if let Some(stream) = self.shared_stream.lock().unwrap().take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
    }

    fn join(self) -> std::io::Result<()> {
        self.handle.join().unwrap()
    }
}

struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn set_new(path: &Path) -> std::io::Result<Self> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { original })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

fn create_search_index(root: &Path) {
    let index_path = root.join("search_index");
    std::fs::create_dir_all(&index_path).expect("create search_index dir");
    let schema = SearchSchema::build();
    let index = Index::create_in_dir(&index_path, schema.schema.clone()).expect("create index");
    let mut writer = index.writer(50_000_000).expect("writer");
    writer
        .add_document(doc!(
            schema.url_field => "https://example.com/",
            schema.title_field => "adapter smoke",
            schema.content_field => "adapter lifecycle test",
            schema.domain_field => "example.com",
            schema.quality_field => "0.5",
            schema.pagerank_field => 0.1f64,
            schema.tfidf_field => 0.1f64
        ))
        .expect("add doc");
    writer.commit().expect("commit");
}

fn wait_for_socket(path: &Path) {
    for _ in 0..50 {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("socket not ready: {:?}", path);
}

#[test]
fn adapter_errors_if_core_missing() {
    let tmp = tempdir().expect("tmpdir");
    create_search_index(tmp.path());
    let _cwd = CwdGuard::set_new(tmp.path()).expect("set cwd");
    let socket_path = tmp.path().join("nerve-missing.sock");
    // ensure no socket exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).ok();
    }

    let result = client::run(socket_path.to_str().unwrap());
    assert!(result.is_err(), "adapter should fail when core is absent");
}

#[test]
#[ignore]
fn adapter_connects_when_core_available() {
    let tmp = tempdir().expect("tmpdir");
    create_search_index(tmp.path());
    let _cwd = CwdGuard::set_new(tmp.path()).expect("set cwd");
    let socket_path = tmp.path().join("nerve-core.sock");
    let core = CoreHarness::start(socket_path.clone());

    wait_for_socket(&socket_path);

    let adapter_handle = thread::spawn({
        let path = socket_path.clone();
        move || client::run(path.to_str().unwrap())
    });

    assert!(
        core.wait_for_connection(Duration::from_millis(500)),
        "adapter should connect to core"
    );

    core.shutdown();
    let adapter_result = adapter_handle.join().expect("adapter join");
    assert!(adapter_result.is_ok(), "adapter should exit cleanly after core shutdown");
    core.join().expect("core join");
}

#[test]
#[ignore]
fn adapter_exits_when_core_shuts_down() {
    let tmp = tempdir().expect("tmpdir");
    create_search_index(tmp.path());
    let _cwd = CwdGuard::set_new(tmp.path()).expect("set cwd");
    let socket_path = tmp.path().join("nerve-core-shutdown.sock");
    let core = CoreHarness::start(socket_path.clone());

    wait_for_socket(&socket_path);

    let adapter_handle = thread::spawn({
        let path = socket_path.clone();
        move || client::run(path.to_str().unwrap())
    });

    assert!(
        core.wait_for_connection(Duration::from_millis(500)),
        "adapter should connect to core"
    );
    core.shutdown();

    let adapter_result = adapter_handle.join().expect("adapter join");
    assert!(adapter_result.is_ok(), "adapter should exit when core shuts down");
    core.join().expect("core join");
}
