use std::sync::Arc;

use futures::future::BoxFuture;

use crate::auth;
use crate::config;
use crate::eventstream;
use crate::http;
use crate::lifecycle;
use crate::notifications;
use crate::oauth2;
use crate::remote;
use crate::remote_files;
use crate::remote_files_dir_pickers;
use crate::repo_config_backup;
use crate::repo_create;
use crate::repo_files;
use crate::repo_files_browsers;
use crate::repo_files_details;
use crate::repo_files_dir_pickers;
use crate::repo_files_list;
use crate::repo_files_move;
use crate::repo_files_read;
use crate::repo_remove;
use crate::repo_space_usage;
use crate::repo_unlock;
use crate::repos;
use crate::runtime;
use crate::secure_storage;
use crate::space_usage;
use crate::store;
use crate::uploads;
use crate::user;

#[allow(dead_code)]
pub struct Vault {
    store: Arc<store::Store>,
    notifications_service: Arc<notifications::NotificationsService>,
    oauth2_service: Arc<oauth2::OAuth2Service>,
    user_service: Arc<user::UserService>,
    uploads_service: Arc<uploads::UploadsService>,
    remote_files_service: Arc<remote_files::RemoteFilesService>,
    remote_files_dir_pickers_service: Arc<remote_files_dir_pickers::RemoteFilesDirPickersService>,
    repos_service: Arc<repos::ReposService>,
    repo_create_service: Arc<repo_create::RepoCreateService>,
    repo_unlock_service: Arc<repo_unlock::RepoUnlockService>,
    repo_remove_service: Arc<repo_remove::RepoRemoveService>,
    repo_config_backup_service: Arc<repo_config_backup::RepoConfigBackupService>,
    repo_space_usage_service: Arc<repo_space_usage::RepoSpaceUsageService>,
    repo_files_service: Arc<repo_files::RepoFilesService>,
    eventstream_service: Arc<eventstream::EventStreamService>,
    repo_files_dir_pickers_service: Arc<repo_files_dir_pickers::RepoFilesDirPickersService>,
    repo_files_browsers_service: Arc<repo_files_browsers::RepoFilesBrowsersService>,
    repo_files_details_service: Arc<repo_files_details::RepoFilesDetailsService>,
    repo_files_move_service: Arc<repo_files_move::RepoFilesMoveService>,
    space_usage_service: Arc<space_usage::SpaceUsageService>,
    lifecycle_service: Arc<lifecycle::LifecycleService>,
}

impl Vault {
    pub fn new(
        base_url: String,
        oauth2_config: oauth2::OAuth2Config,
        http_client: Box<dyn http::HttpClient + Send + Sync>,
        eventstream_websocket_client: Box<dyn eventstream::WebSocketClient + Send + Sync>,
        secure_storage: Box<dyn secure_storage::SecureStorage + Send + Sync>,
        runtime: Box<dyn runtime::Runtime + Send + Sync>,
    ) -> Self {
        let state = store::State {
            config: config::state::ConfigState {
                base_url: base_url.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let store = Arc::new(store::Store::new(state));
        let http_client = Arc::new(http_client);
        let runtime = Arc::new(runtime);
        let secure_storage_service =
            Arc::new(secure_storage::SecureStorageService::new(secure_storage));
        let notifications_service =
            Arc::new(notifications::NotificationsService::new(store.clone()));
        let oauth2_service = Arc::new(oauth2::OAuth2Service::new(
            oauth2_config,
            secure_storage_service.clone(),
            http_client.clone(),
            store.clone(),
        ));
        let auth_provider: Arc<Box<(dyn auth::AuthProvider + Send + Sync + 'static)>> = Arc::new(
            Box::new(oauth2::OAuth2AuthProvider::new(oauth2_service.clone())),
        );
        let remote = Arc::new(remote::Remote::new(
            base_url.clone(),
            http_client.clone(),
            auth_provider.clone(),
        ));
        let user_service = Arc::new(user::UserService::new(remote.clone(), store.clone()));
        let remote_files_service = Arc::new(remote_files::RemoteFilesService::new(
            remote.clone(),
            store.clone(),
        ));
        let remote_files_dir_pickers_service =
            Arc::new(remote_files_dir_pickers::RemoteFilesDirPickersService::new(
                remote_files_service.clone(),
                store.clone(),
            ));
        let repos_service = Arc::new(repos::ReposService::new(remote.clone(), store.clone()));
        let repo_unlock_service = Arc::new(repo_unlock::RepoUnlockService::new(
            repos_service.clone(),
            store.clone(),
        ));
        let repo_remove_service = Arc::new(repo_remove::RepoRemoveService::new(
            repos_service.clone(),
            store.clone(),
        ));
        let repo_config_backup_service = Arc::new(
            repo_config_backup::RepoConfigBackupService::new(repos_service.clone(), store.clone()),
        );
        let repo_space_usage_service = Arc::new(repo_space_usage::RepoSpaceUsageService::new(
            remote_files_service.clone(),
            store.clone(),
        ));
        let repo_files_list_service = Arc::new(repo_files_list::RepoFilesListService::new(
            repos_service.clone(),
            remote_files_service.clone(),
        ));
        let repo_files_read_service = Arc::new(repo_files_read::RepoFilesReadService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            repo_files_list_service.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let repo_files_service = Arc::new(repo_files::RepoFilesService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            repo_files_read_service.clone(),
            store.clone(),
        ));
        let repo_create_service = Arc::new(repo_create::RepoCreateService::new(
            remote.clone(),
            repos_service.clone(),
            remote_files_service.clone(),
            remote_files_dir_pickers_service.clone(),
            store.clone(),
        ));
        let uploads_service = Arc::new(uploads::UploadsService::new(
            repo_files_service.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let eventstream_service = Arc::new(eventstream::EventStreamService::new(
            base_url.clone(),
            eventstream_websocket_client,
            auth_provider.clone(),
            repo_files_service.clone(),
            runtime.clone(),
        ));
        let repo_files_dir_pickers_service =
            Arc::new(repo_files_dir_pickers::RepoFilesDirPickersService::new(
                repo_files_service.clone(),
                store.clone(),
            ));
        let repo_files_browsers_service =
            Arc::new(repo_files_browsers::RepoFilesBrowsersService::new(
                repo_files_service.clone(),
                repo_files_read_service.clone(),
                eventstream_service.clone(),
                store.clone(),
            ));
        let repo_files_details_service =
            Arc::new(repo_files_details::RepoFilesDetailsService::new(
                repo_files_service.clone(),
                repo_files_read_service.clone(),
                eventstream_service.clone(),
                store.clone(),
            ));
        let repo_files_move_service = Arc::new(repo_files_move::RepoFilesMoveService::new(
            repo_files_service.clone(),
            repo_files_dir_pickers_service.clone(),
            store.clone(),
        ));
        let space_usage_service = Arc::new(space_usage::SpaceUsageService::new(
            remote.clone(),
            store.clone(),
        ));
        let lifecycle_service = Arc::new(lifecycle::LifecycleService::new(
            oauth2_service.clone(),
            user_service.clone(),
            repos_service.clone(),
            eventstream_service.clone(),
            space_usage_service.clone(),
            store.clone(),
        ));

        let remote_logout_lifecycle_service = Arc::downgrade(&lifecycle_service);
        remote.set_logout(Box::new(move || {
            if let Some(lifecycle_service) = remote_logout_lifecycle_service.upgrade() {
                lifecycle_service.logout();
            }
        }));

        Self {
            store,
            notifications_service,
            oauth2_service,
            user_service,
            uploads_service,
            remote_files_service,
            remote_files_dir_pickers_service,
            repos_service,
            repo_create_service,
            repo_unlock_service,
            repo_remove_service,
            repo_config_backup_service,
            repo_space_usage_service,
            repo_files_service,
            eventstream_service,
            repo_files_dir_pickers_service,
            repo_files_browsers_service,
            repo_files_details_service,
            repo_files_move_service,
            space_usage_service,
            lifecycle_service,
        }
    }

    // store

    pub fn get_next_id(&self) -> u32 {
        self.store.get_next_id()
    }

    pub fn on(&self, id: u32, events: &[store::Event], callback: Box<dyn Fn() + Send + Sync>) {
        self.store.on(id, events, callback)
    }

    pub fn remove_listener(&self, id: u32) {
        self.store.remove_listener(id)
    }

    pub fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&store::State) -> R,
    {
        self.store.with_state(f)
    }

    // lifecycle

    pub async fn load(&self) -> Result<(), remote::RemoteError> {
        self.lifecycle_service.load().await
    }

    pub fn logout(&self) {
        self.lifecycle_service.logout()
    }

    // notifications

    pub fn notifications_show(&self, message: String) {
        self.notifications_service.show(message)
    }

    pub fn notifications_remove(&self, id: u32) {
        self.notifications_service.remove(id)
    }

    pub fn notifications_remove_all(&self) {
        self.notifications_service.remove_all()
    }

    // oauth2

    pub fn oauth2_start_flow(&self) -> String {
        self.oauth2_service.start_flow()
    }

    pub async fn oauth2_finish_flow_url(
        &self,
        url: &str,
    ) -> Result<(), oauth2::errors::OAuth2Error> {
        self.oauth2_service.finish_flow_url(url).await?;

        self.lifecycle_service
            .on_login()
            .await
            .map_err(|e| match e {
                remote::RemoteError::HttpError(err) => oauth2::errors::OAuth2Error::HttpError(err),
                _ => oauth2::errors::OAuth2Error::Unknown(e.to_string()),
            })?;

        Ok(())
    }

    // user

    pub async fn user_load(&self) -> Result<(), remote::RemoteError> {
        self.user_service.load_user().await
    }

    pub async fn user_ensure_profile_picture(&self) -> Result<(), remote::RemoteError> {
        self.user_service.ensure_profile_picture().await
    }

    // repos

    pub async fn repos_load(&self) -> Result<(), remote::RemoteError> {
        self.repos_service.load_repos().await
    }

    pub fn repos_lock_repo(&self, repo_id: &str) -> Result<(), repos::errors::RepoNotFoundError> {
        self.repos_service.lock_repo(repo_id)
    }

    // repo_create

    pub async fn repo_create_init(&self) {
        self.repo_create_service.init().await
    }

    pub fn repo_create_reset(&self) {
        self.repo_create_service.reset();
    }

    pub fn repo_create_set_location(&self, location: remote_files::state::RemoteFilesLocation) {
        self.repo_create_service.set_location(location)
    }

    pub fn repo_create_set_password(&self, password: String) {
        self.repo_create_service.set_password(password)
    }

    pub fn repo_create_set_salt(&self, salt: Option<String>) {
        self.repo_create_service.set_salt(salt)
    }

    pub fn repo_create_fill_from_rclone_config(&self, config: String) {
        self.repo_create_service.fill_from_rclone_config(config)
    }

    pub async fn repo_create_location_dir_picker_show(&self) -> Result<(), remote::RemoteError> {
        self.repo_create_service.location_dir_picker_show().await
    }

    pub fn repo_create_location_dir_picker_select(&self) {
        self.repo_create_service.location_dir_picker_select()
    }

    pub fn repo_create_location_dir_picker_cancel(&self) {
        self.repo_create_service.location_dir_picker_cancel()
    }

    pub fn repo_create_location_dir_picker_check_create_dir(
        &self,
        name: &str,
    ) -> Result<(), remote::RemoteError> {
        self.repo_create_service
            .location_dir_picker_check_create_dir(name)
    }

    pub async fn repo_create_location_dir_picker_create_dir(
        &self,
        name: &str,
    ) -> Result<(), remote::RemoteError> {
        self.repo_create_service
            .location_dir_picker_create_dir(name)
            .await
    }

    pub async fn repo_create_create(&self) {
        self.repo_create_service.create().await
    }

    // repo_unlock

    pub fn repo_unlock_init(&self, repo_id: &str) {
        self.repo_unlock_service.init(repo_id)
    }

    pub async fn repo_unlock_unlock(
        &self,
        password: &str,
    ) -> Result<(), repos::errors::UnlockRepoError> {
        self.repo_unlock_service.unlock(password).await
    }

    pub fn repo_unlock_destroy(&self, repo_id: &str) {
        self.repo_unlock_service.destroy(repo_id)
    }

    // repo_remove

    pub fn repo_remove_init(&self, repo_id: &str) {
        self.repo_remove_service.init(repo_id)
    }

    pub async fn repo_remove_remove(
        &self,
        password: &str,
    ) -> Result<(), repos::errors::RemoveRepoError> {
        self.repo_remove_service.remove(password).await
    }

    pub fn repo_remove_destroy(&self, repo_id: &str) {
        self.repo_remove_service.destroy(repo_id)
    }

    // repo_config_backup

    pub fn repo_config_backup_init(&self, repo_id: &str) {
        self.repo_config_backup_service.init(repo_id)
    }

    pub async fn repo_config_backup_generate(
        &self,
        password: &str,
    ) -> Result<(), repos::errors::RepoConfigError> {
        self.repo_config_backup_service.generate(password).await
    }

    pub fn repo_config_backup_destroy(&self, repo_id: &str) {
        self.repo_config_backup_service.destroy(repo_id)
    }

    // repo_space_usage

    pub fn repo_space_usage_init(&self, repo_id: &str) {
        self.repo_space_usage_service.init(repo_id)
    }

    pub async fn repo_space_usage_calculate(
        &self,
    ) -> Result<(), repo_space_usage::errors::RepoSpaceUsageError> {
        self.repo_space_usage_service.calculate().await
    }

    pub fn repo_space_usage_destroy(&self, repo_id: &str) {
        self.repo_space_usage_service.destroy(repo_id)
    }

    // repo_files

    pub async fn repo_files_load_files(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_service.load_files(repo_id, path).await
    }

    pub async fn repo_files_get_file_reader(
        self: Arc<Self>,
        file_id: &str,
    ) -> Result<repo_files_read::state::RepoFileReader, repo_files_read::errors::GetFilesReaderError>
    {
        self.repo_files_service
            .clone()
            .get_file_reader(file_id)
            .await
    }

    pub async fn repo_files_delete_file(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<(), repo_files::errors::DeleteFileError> {
        self.repo_files_service.delete_file(repo_id, path).await
    }

    pub fn repo_files_check_rename_file(
        &self,
        repo_id: &str,
        path: &str,
        name: &str,
    ) -> Result<(), repo_files::errors::RenameFileError> {
        self.repo_files_service
            .check_rename_file(repo_id, path, name)
    }

    pub async fn repo_files_rename_file(
        &self,
        repo_id: &str,
        path: &str,
        name: &str,
    ) -> Result<(), repo_files::errors::RenameFileError> {
        self.repo_files_service
            .rename_file(repo_id, path, name)
            .await
    }

    // uploads

    pub async fn uploads_upload(
        &self,
        repo_id: &str,
        parent_path: &str,
        name: &str,
        uploadable: uploads::service::Uploadable,
    ) -> Result<repo_files::state::RepoFilesUploadResult, uploads::errors::UploadError> {
        self.uploads_service
            .clone()
            .upload(repo_id, parent_path, name, uploadable)
            .await
    }

    pub fn uploads_abort_file(&self, id: u32) {
        self.uploads_service.abort_file(id);
    }

    pub fn uploads_abort_all(&self) {
        self.uploads_service.abort_all();
    }

    pub fn uploads_retry_file(&self, id: u32) {
        self.uploads_service.clone().retry_file(id);
    }

    pub fn uploads_retry_all(&self) {
        self.uploads_service.clone().retry_all();
    }

    // repo_files_browsers

    pub fn repo_files_browsers_create(
        &self,
        repo_id: &str,
        path: &str,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_files::errors::LoadFilesError>>,
    ) {
        self.repo_files_browsers_service
            .clone()
            .create(repo_id, path)
    }

    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.repo_files_browsers_service.destroy(browser_id)
    }

    pub async fn repo_files_browsers_set_location(
        &self,
        browser_id: u32,
        repo_id: &str,
        path: &str,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_browsers_service
            .set_location(browser_id, repo_id, path)
            .await
    }

    pub async fn repo_files_browsers_load_files(
        &self,
        browser_id: u32,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_browsers_service
            .load_files(browser_id)
            .await
    }

    pub fn repo_files_browsers_select_file(
        &self,
        browser_id: u32,
        file_id: &str,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.repo_files_browsers_service
            .select_file(browser_id, file_id, extend, range, force)
    }

    pub fn repo_files_browsers_toggle_select_all(&self, browser_id: u32) {
        self.repo_files_browsers_service
            .toggle_select_all(browser_id)
    }

    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.repo_files_browsers_service.clear_selection(browser_id)
    }

    pub fn repo_files_browsers_sort_by(
        &self,
        browser_id: u32,
        field: repo_files::state::RepoFilesSortField,
    ) {
        self.repo_files_browsers_service.sort_by(browser_id, field)
    }

    pub async fn repo_files_browsers_get_selected_reader(
        self: Arc<Self>,
        browser_id: u32,
    ) -> Result<repo_files_read::state::RepoFileReader, repo_files_read::errors::GetFilesReaderError>
    {
        self.repo_files_browsers_service
            .clone()
            .get_selected_reader(browser_id)
            .await
    }

    pub fn repo_files_browsers_check_create_dir(
        &self,
        browser_id: u32,
        name: &str,
    ) -> Result<(), repo_files::errors::CreateDirError> {
        self.repo_files_browsers_service
            .check_create_dir(browser_id, name)
    }

    pub async fn repo_files_browsers_create_dir(
        &self,
        browser_id: u32,
        name: &str,
    ) -> Result<(), repo_files::errors::CreateDirError> {
        self.repo_files_browsers_service
            .create_dir(browser_id, name)
            .await
    }

    pub async fn repo_files_browsers_delete_selected(
        &self,
        browser_id: u32,
    ) -> Result<(), repo_files::errors::DeleteFileError> {
        self.repo_files_browsers_service
            .delete_selected(browser_id)
            .await
    }

    // repo_files_details

    pub fn repo_files_details_create(
        &self,
        repo_id: &str,
        path: &str,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_files::errors::LoadFilesError>>,
    ) {
        self.repo_files_details_service
            .clone()
            .create(repo_id, path)
    }

    pub fn repo_files_details_destroy(&self, details_id: u32) {
        self.repo_files_details_service.destroy(details_id)
    }

    pub async fn repo_files_details_load_content(
        self: Arc<Self>,
        details_id: u32,
    ) -> Result<(), repo_files_read::errors::GetFilesReaderError> {
        self.repo_files_details_service
            .clone()
            .load_content(details_id)
            .await
    }

    pub async fn repo_files_details_get_file_reader(
        self: Arc<Self>,
        details_id: u32,
    ) -> Result<repo_files_read::state::RepoFileReader, repo_files_read::errors::GetFilesReaderError>
    {
        self.repo_files_details_service
            .clone()
            .get_file_reader(details_id)
            .await
    }

    // repo_files_move

    pub async fn repo_files_move_show(
        &self,
        browser_id: u32,
        mode: repo_files_move::state::RepoFilesMoveMode,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_move_service.show(browser_id, mode).await
    }

    pub async fn repo_files_move_move_files(
        &self,
    ) -> Result<(), repo_files::errors::MoveFileError> {
        self.repo_files_move_service.move_files().await
    }

    pub fn repo_files_move_cancel(&self) {
        self.repo_files_move_service.cancel()
    }

    pub fn repo_files_move_check_create_dir(
        &self,
        name: &str,
    ) -> Result<(), repo_files::errors::CreateDirError> {
        self.repo_files_move_service.check_create_dir(name)
    }

    pub async fn repo_files_move_create_dir(
        &self,
        name: &str,
    ) -> Result<(), repo_files::errors::CreateDirError> {
        self.repo_files_move_service.create_dir(name).await
    }

    // remote_files_dir_pickers

    pub async fn remote_files_dir_pickers_load(
        &self,
        picker_id: u32,
    ) -> Result<(), remote::RemoteError> {
        self.remote_files_dir_pickers_service.load(picker_id).await
    }

    pub async fn remote_files_dir_pickers_click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) -> Result<(), remote::RemoteError> {
        self.remote_files_dir_pickers_service
            .click(picker_id, item_id, is_arrow)
            .await
    }

    // repo_files_dir_pickers

    pub async fn repo_files_dir_pickers_click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_dir_pickers_service
            .click(picker_id, item_id, is_arrow)
            .await
    }
}

const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    fn assert_all() {
        assert_send::<Vault>();
        assert_sync::<Vault>();
    }
};
