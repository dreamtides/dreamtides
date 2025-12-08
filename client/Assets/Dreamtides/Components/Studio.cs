#nullable enable
using System.Runtime.CompilerServices;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public sealed class Studio : MonoBehaviour
  {
    [SerializeField]
    internal Camera _studioCamera = null!;

    [SerializeField]
    internal Transform _subjectPosition = null!;

    [SerializeField]
    internal Transform _farSubjectPosition = null!;

    public Camera StudioCamera => _studioCamera;
    public Transform SubjectPosition => _subjectPosition;
    public Transform FarSubjectPosition => _farSubjectPosition;
  }
}
