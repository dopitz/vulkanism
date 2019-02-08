use vk;

pub struct Renderpass {

}

impl Renderpass {
}


//#pragma once
//
//#include "math/vec2.h"
//#include "util/nullable.h"
//#include "vk/device/h_device.h"
//
//#include "vulkan/vulkan.h"
//
//namespace v2::vk {
//  class renderpass {
//    friend class renderpass_factory;
//    renderpass(h_device d, VkRenderPass pass);
//
//  public:  // ====> CONSTUCTOR
//    ~renderpass();
//
//    renderpass(renderpass&& other) = default;
//    renderpass& operator=(renderpass&& other) = default;
//    renderpass(const renderpass& other) = delete;
//    renderpass& operator=(const renderpass& other) = delete;
//
//  public:  // ====> FUNCTIONS
//    operator bool() const;
//
//    h_device get_device() const;
//    VkRenderPass get_handle() const;
//
//
//  private:  // ===> VARIABLES
//    h_device _device;
//    util::nullable<VkRenderPass> _handle;
//  };
//}  // namespace v2::vk
