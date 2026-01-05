#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;

/// <summary>
/// Configuration for fixing foot rotations for a specific animation clip.
/// </summary>
[Serializable]
public class ClipFootRotation
{
    [Tooltip("Name of the animation clip (e.g., 'Anim_Fly_Normal_Idle')")]
    public string clipName = "";

    [Tooltip("Local rotation to apply to the left foot (in degrees)")]
    public Vector3 leftFootRotation = new(0f, 0f, 0f);

    [Tooltip("Local rotation to apply to the right foot (in degrees)")]
    public Vector3 rightFootRotation = new(0f, 0f, 0f);
}

/// <summary>
/// Fixes foot bone rotations during flying animations by overriding them to a natural downward pose.
/// This solves issues with humanoid animation retargeting that can cause feet to rotate backwards.
/// Configure different foot rotations per animation clip.
/// </summary>
[RequireComponent(typeof(Animator))]
public class FlyingFootFixer : MonoBehaviour
{
    [Header("Clip-Based Foot Rotation Configurations")]
    [Tooltip("List of animation clips and their corresponding foot rotation fixes")]
    [SerializeField] private List<ClipFootRotation> clipConfigurations = new();

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
        if (animator == null || leftFoot == null || rightFoot == null)
            return;

        // Find the configuration for the current animation clip
        ClipFootRotation? config = GetCurrentClipConfiguration();
        if (config == null)
            return;

        // Apply the foot rotation fix
        FixFootRotation(leftFoot, config.leftFootRotation);
        FixFootRotation(rightFoot, config.rightFootRotation);
    }

    private ClipFootRotation? GetCurrentClipConfiguration()
    {
        if (animator == null)
            return null;

        // Get the current animation clip info for all layers
        for (int layer = 0; layer < animator.layerCount; layer++)
        {
            AnimatorClipInfo[] clipInfos = animator.GetCurrentAnimatorClipInfo(layer);
            foreach (AnimatorClipInfo clipInfo in clipInfos)
            {
                string clipName = clipInfo.clip.name;

                // Find matching configuration
                foreach (ClipFootRotation config in clipConfigurations)
                {
                    if (config.clipName == clipName)
                    {
                        return config;
                    }
                }
            }
        }

        return null;
    }

    private void FixFootRotation(Transform foot, Vector3 targetRotationEuler)
    {
        // Convert euler angles to quaternion and apply directly
        foot.localRotation = Quaternion.Euler(targetRotationEuler);
    }
}
