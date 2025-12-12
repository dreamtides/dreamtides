#nullable enable
using System.Runtime.CompilerServices;
using Dreamtides.Schema;
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

    [SerializeField]
    internal StudioType _studioType;

    [SerializeField]
    internal bool _isFar;

    public Camera StudioCamera => _studioCamera;
    public Transform SubjectPosition => _subjectPosition;
    public Transform FarSubjectPosition => _farSubjectPosition;

    public StudioType StudioType
    {
      get => _studioType;
      set => _studioType = value;
    }

    public bool IsFar
    {
      get => _isFar;
      set => _isFar = value;
    }
  }
}
