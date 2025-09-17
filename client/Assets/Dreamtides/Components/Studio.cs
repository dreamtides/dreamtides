#nullable enable
using UnityEngine;

namespace Dreamtides.Components
{
  public sealed class Studio : MonoBehaviour
  {
    [SerializeField]
    Camera _studioCamera = null!;

    [SerializeField]
    Transform _subjectPosition = null!;

    [SerializeField]
    Transform _farSubjectPosition = null!;

    public Camera StudioCamera => _studioCamera;
    public Transform SubjectPosition => _subjectPosition;
    public Transform FarSubjectPosition => _farSubjectPosition;
  }
}
