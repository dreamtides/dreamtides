using System.Runtime.InteropServices;

static class Plugin
{
    public static int ReturnTwo()
    {
        return dreamcaller_return_two();
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("plugin")]
#endif
    public static extern int dreamcaller_return_two();
}
