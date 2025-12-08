#nullable enable

using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class BattleCameraBounds : MonoBehaviour
  {
    [SerializeField]
    internal Transform _bottomLeftAnchor = null!;
    public Transform BottomLeftAnchor => _bottomLeftAnchor;

    [SerializeField]
    internal Transform _topLeftAnchor = null!;
    public Transform TopLeftAnchor => _topLeftAnchor;
  }
}
