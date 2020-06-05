
#[cfg(test)]
mod tests {
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
        println!("{:?}", _adapter.get_info() );


    }
    #[test]
    #[should_panic]
    fn it_works() {
        futures::executor::block_on(get_adapter_info());
    }
}