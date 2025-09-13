#nullable enable

using System;
using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Services
{
  public class DreamscapeService : Service
  {
    [SerializeField] ObjectLayout _tmpSiteDeckLayout = null!;

    public ObjectLayout SiteDeckLayout(Guid siteId)
    {
      return _tmpSiteDeckLayout;
    }
  }
}