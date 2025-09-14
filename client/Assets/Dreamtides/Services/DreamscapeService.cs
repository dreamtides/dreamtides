#nullable enable

using System;
using DG.Tweening;
using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Services
{
  public class DreamscapeService : Service
  {
    [SerializeField] ObjectLayout _tmpSiteDeckLayout = null!;

    public void ApplySiteLayouts(Sequence? sequence)
    {
      _tmpSiteDeckLayout.ApplyLayout(sequence);
    }

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      return _tmpSiteDeckLayout;
    }
  }
}