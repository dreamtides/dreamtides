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

    [SerializeField]
    ObjectLayout _tmpMerchantPositionLayout = null!;

    public void ApplyLayouts(Sequence? sequence)
    {
      Registry.DreamscapeLayout.DraftPickLayout.ApplyLayout(sequence);
      Registry.DreamscapeLayout.ShopLayout.ApplyLayout(sequence);

      _tmpSiteDeckLayout.ApplyLayout(sequence);
      _tmpMerchantPositionLayout.ApplyLayout(sequence);
    }

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      return _tmpSiteDeckLayout;
    }

    public ObjectLayout MerchantPositionLayout(Guid merchantId)
    {
      return _tmpMerchantPositionLayout;
    }
  }
}
