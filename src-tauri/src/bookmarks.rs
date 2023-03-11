use kolekk_types::{ByteArrayFile, DragDropPaste};
use tauri::State;

use crate::{bad_error::Error, database::AppDatabase, config::AppConfig};




#[tauri::command]
pub async fn save_bookmark(
    data: DragDropPaste<ByteArrayFile>,
    config: State<'_, AppConfig>,
    db: State<'_, AppDatabase>,
) -> Result<(), Error> {
    todo!()
}
