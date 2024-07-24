use crate::{
    create_hook,
    cs2::{self},
    get_original_fn,
    utils::{self, hook_system, render},
};

use anyhow::bail;

use windows::{
    core::HRESULT,
    Win32::Graphics::Dxgi::{Common::DXGI_FORMAT, IDXGISwapChain},
};

extern "system" fn hk_present(
    swapchain: IDXGISwapChain,
    sync_interval: u32,
    flags: u32,
) -> HRESULT {
    get_original_fn!(hk_present, original_fn, (IDXGISwapChain, u32, u32), HRESULT);

    render::dx11::init_from_swapchain(&swapchain);

    original_fn(swapchain, sync_interval, flags)
}

extern "system" fn hk_resize_buffers(
    swapchain: IDXGISwapChain,
    buffer_count: u32,
    width: u32,
    height: u32,
    new_format: DXGI_FORMAT,
    swapchain_flags: u32,
) -> HRESULT {
    get_original_fn!(
        hk_resize_buffers,
        original_fn,
        (IDXGISwapChain, u32, u32, u32, DXGI_FORMAT, u32),
        HRESULT
    );

    let mut renderer = render::dx11::DX11
        .get()
        .expect(&"dx11 renderer is not initialized while resizing buffers")
        .lock();

    renderer
        .resize_buffers(&swapchain, || {
            original_fn(swapchain.clone(), buffer_count, width, height, new_format, swapchain_flags)
        })
        .expect(&"could not resize buffers")
}

unsafe extern "system" fn hk_create_move(
    a1: *mut f32,
    a2: u64,
    a3: i8,
    a4: u64,
    a5: u64,
    a6: u64,
) -> u64 {
    // TODO: Your custom logic here
    println!("hk_create_move called");

    get_original_fn!(hk_create_move, original_fn, (*mut f32, u64, i8, u64, u64, u64), u64);

    println!("{}", cs2::interfaces::engine_client().is_in_game());

    original_fn(a1, a2, a3, a4, a5, a6)
}

/// Initializes hooks for various modules in the game.
///
/// This function initializes the MinHook library and sets up hooks for specific functions.
pub fn initialize_hooks() -> anyhow::Result<()> {
    // Initialize MinHook
    if let Err(status) = utils::hook_system::initialize_minhook() {
        bail!("Failed to initialize MinHook: {}", status);
    }

    let create_move_target = cs2::modules::client().find_seq_of_bytes("48 8B C4 4C 89 48 20 55");
    let present_target = cs2::modules::gameoverlayrenderer64().find_seq_of_bytes(
        "48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 57 41 56 41 57 48 83 EC 20 41 8B E8",
    );
    let resize_buffers_target = cs2::modules::gameoverlayrenderer64().find_seq_of_bytes(
        "48 89 5C 24 08 48 89 6C 24 10 48 89 74 24 18 57 41 56 41 57 48 83 EC 30 44",
    );

    create_hook!(create_move_target, hk_create_move);
    create_hook!(present_target, hk_present);
    create_hook!(resize_buffers_target, hk_resize_buffers);

    Ok(())
}