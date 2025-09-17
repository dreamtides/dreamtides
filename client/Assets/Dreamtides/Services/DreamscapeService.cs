#nullable enable

using System;
using DG.Tweening;
using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Services
{
  public class DreamscapeService : Service
  {
    [SerializeField]
    ObjectLayout _tmpSiteDeckLayout = null!;

    public void ApplyLayouts(Sequence? sequence)
    {
      Registry.DreamscapeLayout.DraftPickLayout.ApplyLayout(sequence);
      _tmpSiteDeckLayout.ApplyLayout(sequence);
    }

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      return _tmpSiteDeckLayout;
    }
  }
}
