using System;
using FanControl.Plugins;

namespace fan_control_arc
{
    public class Main : IPlugin
    {
        public void Initialize()
        {
            FanControlArcLib.InitApi();
        }

        public void Load(IPluginSensorsContainer _container)
        {
            foreach (var sensor in FanControlArcLib.Sensors())
            {
                _container.TempSensors.Add(new CoreTempSensors
                {
                    name = sensor.name,
                    id = sensor.id,
                });
                _container.TempSensors.Add(new VRamTempSensors
                {
                    name = sensor.name,
                    id = sensor.id,
                });
                /*_container.ControlSensors.Add(new FanControlSensors()
                {
                    name = sensor.name,
                    id = sensor.id,
                });*/
            }
        }

        public void Close()
        {
        }

        public string Name
        {
            get => "FanControlArc";
        }
    }

    class CoreTempSensors : IPluginSensor
    {
        public FanControlArcLib.Luid id;
        public String name;
        
        public void Update()
        {
        }

        public string Id
        {
            get => Name;
        }

        public string Name
        {
            get => $"Core - {name}";
        }

        public float? Value
        {
            get => (float)FanControlArcLib.GetGpuTemp(id);
        }
    }

    class VRamTempSensors : IPluginSensor
    {
        public FanControlArcLib.Luid id;
        public String name;
        
        public void Update()
        {
        }

        public string Id
        {
            get => Name;
        }

        public string Name
        {
            get => $"VRam - {name}";
        }

        public float? Value
        {
            get => (float)FanControlArcLib.GetVramTemp(id);
        }
    }

    class FanControlSensors: IPluginControlSensor
    {
        public FanControlArcLib.Luid id;
        public String name;
        
        public void Update()
        {
            
        }

        public string Id
        {
            get => Name;
        }

        public string Name
        {
            get => $"Fan - {name}";
        }

        public float? Value
        {
            get => (float)FanControlArcLib.GetFanSpeed(id);
        }
        public void Set(float val)
        {
            FanControlArcLib.SetFanSpeed(id, (int)val);
        }

        public void Reset()
        {
            
        }
    }
}