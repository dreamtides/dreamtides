#nullable enable

using System.Collections.Generic;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Layout
{
  public sealed class TemptingOfferObjectLayout : SitePickObjectLayout
  {
    const int ObjectsPerRow = 2;
    const string DefaultButtonLabel = "Accept";

    [SerializeField]
    float _buttonVerticalOffset;

    [SerializeField]
    float _buttonDepthOffset;

    [SerializeField]
    DisplayableButton _acceptButtonPrefab = null!;

    [SerializeField]
    Transform? _buttonContainer;

    readonly List<DisplayableButton> _acceptButtons = new();
    readonly Dictionary<long, ButtonView> _buttonViewsByOfferNumber = new();
    readonly ButtonView _defaultButtonView = new() { Label = DefaultButtonLabel };

    public void SetOfferActions(IReadOnlyList<TemptingOfferAction>? actions)
    {
      _buttonViewsByOfferNumber.Clear();
      if (actions != null)
      {
        for (var i = 0; i < actions.Count; i++)
        {
          var action = actions[i];
          if (action.Button != null)
          {
            _buttonViewsByOfferNumber[action.Number] = action.Button;
          }
        }
      }
      PositionAcceptButtons();
    }

    protected override void OnBecameNonEmpty()
    {
      base.OnBecameNonEmpty();
      PositionAcceptButtons();
    }

    protected override void OnBecameEmpty()
    {
      base.OnBecameEmpty();
      for (var i = 0; i < _acceptButtons.Count; i++)
      {
        _acceptButtons[i].gameObject.SetActive(false);
      }
    }

    protected override void OnAppliedLayout()
    {
      base.OnAppliedLayout();
      PositionAcceptButtons();
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var effectiveCount = GetEffectiveCount(count);
      if (effectiveCount <= 0)
      {
        return transform.position;
      }
      var rowCount = GetRowCount(effectiveCount);
      var rowIndex = index / ObjectsPerRow;
      var columnIndex = index % ObjectsPerRow;
      var rowItemCount = GetRowItemCount(rowIndex, effectiveCount);
      var localX = GetHorizontalOffset(columnIndex, rowItemCount);
      var localY = GetVerticalOffset(rowIndex, rowCount);
      return transform.position + transform.right * localX + transform.up * localY;
    }

    void PositionAcceptButtons()
    {
      var rowCount = GetRowCount(GetEffectiveCount(Objects.Count));
      if (_acceptButtonPrefab && _acceptButtons.Count < rowCount)
      {
        EnsureButtonInstances(rowCount);
      }
      for (var rowIndex = 0; rowIndex < _acceptButtons.Count; rowIndex++)
      {
        var button = _acceptButtons[rowIndex];
        var shouldBeActive = rowIndex < rowCount;
        button.gameObject.SetActive(shouldBeActive);
        if (!shouldBeActive)
        {
          continue;
        }
        var rowCenter = CalculateRowCenter(rowIndex, rowCount);
        var buttonPosition =
          rowCenter - transform.up * _buttonVerticalOffset + transform.forward * _buttonDepthOffset;
        button.transform.SetPositionAndRotation(buttonPosition, transform.rotation);
        var offerNumber = GetOfferNumberForRow(rowIndex);
        var view = ResolveButtonView(offerNumber);
        button.SetView(Registry, view);
      }
    }

    void EnsureButtonInstances(int requiredCount)
    {
      var parent = _buttonContainer ? _buttonContainer : transform;
      while (_acceptButtons.Count < requiredCount)
      {
        var button = Instantiate(_acceptButtonPrefab, parent);
        button.ExcludeFromLayout = true;
        button.gameObject.SetActive(false);
        _acceptButtons.Add(button);
      }
    }

    int GetRowCount(int count)
    {
      if (count <= 0)
      {
        return 0;
      }
      return (count + ObjectsPerRow - 1) / ObjectsPerRow;
    }

    int GetRowItemCount(int rowIndex, int totalCount)
    {
      var startIndex = rowIndex * ObjectsPerRow;
      var remaining = totalCount - startIndex;
      return Mathf.Clamp(remaining, 0, ObjectsPerRow);
    }

    float GetHorizontalOffset(int columnIndex, int rowItemCount)
    {
      if (rowItemCount <= 1)
      {
        return 0f;
      }
      return columnIndex == 0 ? -HorizontalSpacing / 2f : HorizontalSpacing / 2f;
    }

    float GetVerticalOffset(int rowIndex, int rowCount)
    {
      if (rowCount <= 1)
      {
        return 0f;
      }
      var totalHeight = VerticalSpacing * (rowCount - 1);
      return totalHeight / 2f - rowIndex * VerticalSpacing;
    }

    Vector3 CalculateRowCenter(int rowIndex, int rowCount)
    {
      var localY = GetVerticalOffset(rowIndex, rowCount);
      return transform.position + transform.up * localY;
    }

    long? GetOfferNumberForRow(int rowIndex)
    {
      var objectIndex = rowIndex * ObjectsPerRow;
      if (objectIndex >= Objects.Count)
      {
        return null;
      }
      if (Objects[objectIndex] is Card card)
      {
        var view = card.NullableCardView;
        var position = view?.Position;
        var location = position?.Position;
        var positionClass = location?.PositionClass;
        var offerPosition = positionClass?.TemptingOfferDisplay;
        return offerPosition?.Number;
      }
      return null;
    }

    ButtonView ResolveButtonView(long? offerNumber)
    {
      if (
        offerNumber.HasValue
        && _buttonViewsByOfferNumber.TryGetValue(offerNumber.Value, out var view)
      )
      {
        return view;
      }
      return _defaultButtonView;
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var halfLayoutX = CardWidth / 2f + HorizontalSpacing / 2f;
      var halfLayoutY = CardHeight / 2f + VerticalSpacing / 2f;
      var right = transform.right;
      var upAxis = transform.up;
      Gizmos.DrawSphere(center, 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
    }
  }
}
