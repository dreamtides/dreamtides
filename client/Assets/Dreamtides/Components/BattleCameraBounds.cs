#nullable enable

using UnityEngine;

namespace Dreamtides.Components
{
  public class BattleCameraBounds : MonoBehaviour
  {
    [SerializeField]
    Transform _bottomLeftAnchor = null!;
    public Transform BottomLeftAnchor => _bottomLeftAnchor;

    [SerializeField]
    Transform _topLeftAnchor = null!;
    public Transform TopLeftAnchor => _topLeftAnchor;
  }
}
