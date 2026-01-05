#nullable enable
using UnityEngine;

/// <summary>
/// Fixes foot bone rotations during flying animations by overriding them to a natural downward pose.
/// This solves issues with humanoid animation retargeting that can cause feet to rotate backwards.
/// </summary>
[RequireComponent(typeof(Animator))]
public class FlyingFootFixer : MonoBehaviour
{
    [Header("Foot Rotation Settings")]
    [Tooltip("Enable/disable the foot correction")]
    [SerializeField] private bool applyFix = true;

    [Tooltip("Local rotation to apply to the left foot (in degrees)")]
    [SerializeField] private Vector3 leftFootRotation = new(0f, 0f, 0f);

    [Tooltip("Local rotation to apply to the right foot (in degrees)")]
    [SerializeField] private Vector3 rightFootRotation = new(0f, 0f, 0f);

    [Tooltip("How much to blend between original and fixed rotation. 1 = fully fixed, 0 = original animation")]
    [Range(0f, 1f)]
    [SerializeField] private float blendWeight = 1f;

    [Header("Optional: Animation State Filtering")]
    [Tooltip("If set, only apply fix when this animation state is active (leave empty to always apply)")]
    [SerializeField] private string? animationStateName;

    [Tooltip("Which animator layer to check for the animation state (default: 0)")]
    [SerializeField] private int animatorLayer = 0;

    private Animator? animator;
    private Transform? leftFoot;
    private Transform? rightFoot;

    private void Start()
    {
        animator = GetComponent<Animator>();

        if (animator == null)
        {
            Debug.LogError($"FlyingFootFixer on {gameObject.name} requires an Animator component!", this);
            enabled = false;
            return;
        }

        // Get the humanoid foot bones
        leftFoot = animator.GetBoneTransform(HumanBodyBones.LeftFoot);
        rightFoot = animator.GetBoneTransform(HumanBodyBones.RightFoot);

        if (leftFoot == null || rightFoot == null)
        {
            Debug.LogError($"FlyingFootFixer on {gameObject.name} could not find foot bones. Is the avatar configured as Humanoid?", this);
            enabled = false;
        }
    }

    private void LateUpdate()
    {
        if (!applyFix || animator == null || leftFoot == null || rightFoot == null)
            return;

        // Check if we should apply the fix based on animation state
        if (!ShouldApplyFix())
            return;

        // Apply the foot rotation fix
        FixFootRotation(leftFoot, leftFootRotation);
        FixFootRotation(rightFoot, rightFootRotation);
    }

    private bool ShouldApplyFix()
    {
        // If no specific animation state is specified, always apply
        if (string.IsNullOrEmpty(animationStateName))
            return true;

        if (animator == null)
            return false;

        // Check if the specified animation state is currently active
        AnimatorStateInfo stateInfo = animator.GetCurrentAnimatorStateInfo(animatorLayer);
        return stateInfo.IsName(animationStateName);
    }

    private void FixFootRotation(Transform foot, Vector3 targetRotationEuler)
    {
        // Convert euler angles to quaternion
        Quaternion targetRotation = Quaternion.Euler(targetRotationEuler);

        // Blend between original and target rotation
        if (blendWeight < 1f)
        {
            foot.localRotation = Quaternion.Slerp(foot.localRotation, targetRotation, blendWeight);
        }
        else
        {
            foot.localRotation = targetRotation;
        }
    }
}
