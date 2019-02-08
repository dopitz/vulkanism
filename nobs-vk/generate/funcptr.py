import typeid as tid

def get_ptrs():
    return [
        tid.maketype("PFN_vkVoidFunction", "funcpointer", "extern \"system\" fn() -> ()", []),
        tid.maketype("PFN_vkInternalAllocationNotification", 
            "funcpointer", 
            "extern \"system\" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope) -> ()", 
            ["VkInternalAllocationType", "VkSystemAllocationScope"]),
        tid.maketype("PFN_vkInternalFreeNotification", 
            "funcpointer", 
            "extern \"system\" fn(*mut c_void, usize, InternalAllocationType, SystemAllocationScope) -> ()", 
            ["VkInternalAllocationType", "VkSystemAllocationScope"]),
        tid.maketype("PFN_vkReallocationFunction", 
            "funcpointer", 
            "extern \"system\" fn(*mut c_void, *mut c_void, usize, usize, SystemAllocationScope) -> *mut c_void", 
            ["VkSystemAllocationScope"]),
        tid.maketype("PFN_vkAllocationFunction", 
            "funcpointer", 
            "extern \"system\" fn(*mut c_void, usize, usize, SystemAllocationScope) -> *mut c_void", 
            ["VkSystemAllocationScope"]),
        tid.maketype("PFN_vkFreeFunction", 
            "funcpointer", 
            "extern \"system\" fn(*mut c_void, *mut c_void) -> ()", 
            []),
        tid.maketype("PFN_vkDebugReportCallbackEXT", 
            "funcpointer", 
            "extern \"system\" fn(flags: DebugReportFlagsEXT, objectType: DebugReportObjectTypeEXT, object: u64, location: usize, messageCode: i32, pLayerPrefix: *mut c_char, pMessage: *mut c_char, pUserData: *mut c_void) -> Bool32", 
            ["VkDebugReportFlagsEXT", "VkDebugReportObjectTypeEXT"]),
        tid.maketype("PFN_vkDebugUtilsMessengerCallbackEXT", 
            "funcpointer", 
            "extern \"system\" fn(messageSeverity: DebugUtilsMessageSeverityFlagBitsEXT, messageTypes: DebugUtilsMessageTypeFlagsEXT, pCallbackData: *const DebugUtilsMessengerCallbackDataEXT, pUserData: *mut c_void) -> Bool32", 
            ["VkDebugUtilsMessageSeverityFlagBitsEXT", "VkDebugUtilsMessageTypeFlagsEXT", "VkDebugUtilsMessengerCallbackDataEXT"])
        ]


def write_ptr(t):
    return "#[doc(hidden)] pub type " + t.name + " = " + t.type + ";\n"

