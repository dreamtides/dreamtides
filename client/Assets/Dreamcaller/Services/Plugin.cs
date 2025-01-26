#nullable enable

using System.Runtime.InteropServices;
using Dreamcaller.Schema;

static class Plugin
{
    const int BufferSize = 1_000_000;

    public static int ReturnTwo()
    {
        return dreamcaller_return_two();
    }

    public static BattleView GetScene(int scene)
    {
        byte[] response = new byte[BufferSize];
        int responseLength = dreamcaller_get_scene(scene, response, BufferSize);
        var json = System.Text.Encoding.UTF8.GetString(response, 0, responseLength);
        return BattleView.FromJson(json);
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_return_two();

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_get_scene(int scene, [Out] byte[] response, int responseLength);
}
