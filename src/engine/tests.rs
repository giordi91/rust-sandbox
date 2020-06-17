#[cfg(test)]
mod tests {

    use super::super::handle;

    async fn get_adapter_info() {
        let _instance = wgpu::Instance::new();
        let _adapter = _instance
            .request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: None,
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap();
        println!("{:?}", _adapter.get_info());
    }
    #[test]
    #[should_panic]
    fn it_works() {
        futures::executor::block_on(get_adapter_info());
    }

    //handles
    #[test]
    fn basic_handle_tests() {
        let test_handle = handle::ResourceHandle::new(handle::ResourceHandleType::Shader, 43);
        assert_eq!(test_handle.get_type(), handle::ResourceHandleType::Shader);
        assert_eq!(test_handle.get_value(), 43);

        let test_handle_mesh = handle::ResourceHandle::new(handle::ResourceHandleType::Mesh, 9999);
        assert_eq!(
            test_handle_mesh.get_type(),
            handle::ResourceHandleType::Mesh
        );
        assert_eq!(test_handle_mesh.get_value(), 9999);

        let test_handle_invalid =
            handle::ResourceHandle::new(handle::ResourceHandleType::Invalid, 3243);
        assert_eq!(
            test_handle_invalid.get_type(),
            handle::ResourceHandleType::Invalid
        );
        assert_eq!(test_handle_invalid.get_value(), 3243);
    }
}
