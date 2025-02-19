#nullable enable

using System.Collections;
using AmazingAssets.AdvancedDissolve;
using Dreamcaller.Schema;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class DissolveEffect : MonoBehaviour
  {
    [SerializeField] MeshRenderer _meshRenderer;
    [SerializeField] Material _dissolveMaterial;
    [SerializeField] Color _edgeColor;
    [SerializeField] float _dissolveSpeed = 0.1f;
    Material? _originalMaterial;
    bool _running = false;
    float _clipValue = 0;
    bool _reverse = false;

    public IEnumerator StartDissolve(DissolveCardCommand command)
    {
      _reverse = command.Reverse;
      // _originalMaterial = _meshRenderer.material;

      var material = Instantiate(_dissolveMaterial);
      material.mainTexture = _meshRenderer.material.mainTexture;
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
      _meshRenderer.material = material;
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
        _meshRenderer.material,
        AdvancedDissolveProperties.Cutout.Standard.Property.Clip,
        _clipValue);

      if (_clipValue >= 1 || _clipValue <= 0)
      {
        if (_reverse)
        {
          // _meshRenderer.material = _originalMaterial;
        }
        _running = false;
      }
    }
  }
}