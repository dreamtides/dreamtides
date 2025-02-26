#nullable enable

using System.Collections;
using AmazingAssets.AdvancedDissolve;
using Dreamcaller.Schema;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DissolveEffect : MonoBehaviour
  {
    [SerializeField] Renderer _target = null!;
    [SerializeField] Material _dissolveMaterial = null!;
    [SerializeField] Color _edgeColor;
    [SerializeField] float _dissolveSpeed = 0.1f;
    [SerializeField] Material _originalMaterial = null!;
    bool _running = false;
    float _clipValue = 0;
    bool _reverse = false;

    public IEnumerator StartDissolve(DissolveCardCommand command)
    {
      _reverse = command.Reverse;
      if (!_reverse)
      {
        _originalMaterial = _target.material;
      }

      var material = Instantiate(_dissolveMaterial);
      material.mainTexture = _target.material.mainTexture;
      AdvancedDissolveKeywords.SetKeyword(
          material,
          AdvancedDissolveKeywords.State.Enabled, true);
      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Cutout.Standard.Property.Clip, _reverse ? 1f : 0f);
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.Color, _edgeColor);
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.ColorIntensity,
          Random.Range(5f, 8f));
      AdvancedDissolveProperties.Edge.Base.UpdateLocalProperty(
          material,
          AdvancedDissolveProperties.Edge.Base.Property.Shape,
          AdvancedDissolveProperties.Edge.Base.Shape.Smoother);
      _target.material = material;
      _running = true;
      yield return new WaitUntil(() => !_running);
    }

    void Update()
    {
      if (!_running)
      {
        return;
      }

      _clipValue += Time.deltaTime * _dissolveSpeed * (_reverse ? -1 : 1);
      _clipValue = Mathf.Clamp01(_clipValue);
      AdvancedDissolveProperties.Cutout.Standard.UpdateLocalProperty(
        _target.material,
        AdvancedDissolveProperties.Cutout.Standard.Property.Clip,
        _clipValue);

      if (_clipValue >= 1 || _clipValue <= 0)
      {
        if (_reverse)
        {
          _target.material = _originalMaterial;
        }
        _running = false;
      }
    }
  }
}