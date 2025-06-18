#nullable enable

using UnityEngine;

namespace Dreamtides.Components
{
  public class PlayerStatusAnimations : MonoBehaviour
  {
    [SerializeField] StudioType _studioType;
    [SerializeField] Registry _registry;
    [SerializeField] float _lowerBoundSeconds;
    [SerializeField] float _upperBoundSeconds;
    [SerializeField] float _loopSeconds;

    /**
Default State:
IDL_Base


Animation List:

IDL_ArmsFolded_Casual_Enter
IDL_ArmsFolded_Casual_Loop
IDL_ArmsFolded_Casual_Exit
IDL_Bored_SlumpBack
IDL_Bored_SwingArms
IDL_HandsOnHips_Base_Enter
IDL_HandsOnHips_Base_Loop
IDL_HandsOnHips_Base_Exit
IDL_HeadNod_Large
IDL_HeadNod_Small
IDL_HeadShake_Disappointed
IDL_HeadShake_Large
IDL_HeadShake_Small
IDL_Inspect_Hands
IDL_Lean_B_Base_Enter
IDL_Lean_B_Base_Loop
IDL_Lean_B_Base_Exit
IDL_Lean_L_Base_Enter
IDL_Lean_L_Base_Loop
IDL_Lean_L_Base_Exit
IDL_Look_R
IDL_Look_L_Scared
IDL_Plead_F
IDL_PointHand_Thumb_L
IDL_PointHand_Index_F
IDL_Posture_Aggressive_Enter
IDL_Posture_Aggressive_Loop
IDL_Posture_Aggressive_Exit
IDL_Idles_Posture_Slumped_Enter
IDL_Idles_Posture_Slumped_Loop
IDL_Idles_Posture_Slumped_Exit
IDL_Stretch_Arms
IDL_Stretch_Shoulders
IDL_Thoughtful_L_ChinScratch
IDL_Thoughtful_R_ChinScratch
IDL_WeightShift_L
IDL_WeightShift_R
IDL_Yawn_Masc

*/
  }
}