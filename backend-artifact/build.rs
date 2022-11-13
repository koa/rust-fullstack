use static_files::resource_dir;

fn main() -> std::io::Result<()> {
    resource_dir("../yew-app/dist").build()
}
