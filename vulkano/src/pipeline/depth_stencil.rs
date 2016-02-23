
pub struct DepthStencil {
    depth_write: bool,
    depth_compare: Compare,
    depth_bounds_test: bool,
}



    VkBool32                                    depthTestEnable;
    VkBool32                                    depthWriteEnable;
    VkCompareOp                                 depthCompareOp;
    VkBool32                                    depthBoundsTestEnable;
    VkBool32                                    stencilTestEnable;
    VkStencilOpState                            front;
    VkStencilOpState                            back;
    float                                       minDepthBounds;
    float                                       maxDepthBounds;

typedef struct {
    VkStencilOp                                 stencilFailOp;
    VkStencilOp                                 stencilPassOp;
    VkStencilOp                                 stencilDepthFailOp;
    VkCompareOp                                 stencilCompareOp;
    uint32_t                                    stencilCompareMask;
    uint32_t                                    stencilWriteMask;
    uint32_t                                    stencilReference;
} VkStencilOpState;

/// Specifies how two values should be compared to decide whether a test passes or fails.
///
/// Used for both depth testing and stencil testing.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum Compare {
    /// The test never passes.
    Never => vk::COMPARE_OP_NEVER,
    /// The test passes if `value < reference_value`.
    Less => vk::COMPARE_OP_LESS,
    /// The test passes if `value == reference_value`.
    Equal => vk::COMPARE_OP_EQUAL,
    /// The test passes if `value <= reference_value`.
    LessOrEqual => vk::COMPARE_OP_LESS_OR_EQUAL,
    /// The test passes if `value > reference_value`.
    Greater => vk::COMPARE_OP_GREATER,
    /// The test passes if `value != reference_value`.
    NotEqual => vk::COMPARE_OP_NOT_EQUAL,
    /// The test passes if `value >= reference_value`.
    GreaterOrEqual => vk::COMPARE_OP_GREATER_OR_EQUAL,
    /// The test always passes.
    Always => vk::COMPARE_OP_ALWAYS,
}
