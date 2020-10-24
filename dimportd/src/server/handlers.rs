use crate::Importer;

pub fn status(importer: &Importer) -> String {
    return importer.state.changed_files.join("\n");
}
