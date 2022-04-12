using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using OrdoProject;
using IHost host = Host.CreateDefaultBuilder(args)
    .UseWindowsService(options =>
    {
        options.ServiceName = "Ordo Service";
    })
    .ConfigureServices(services =>
    {
        services.AddSingleton<OrdoService>();
        services.AddHostedService<Worker>();
    })
    .Build();

await host.RunAsync();

//namespace OrdoProject
//{
//    public class Program
//    {
//        public static void Main(string[] args)
//        {
//            CreateHostBuilder(args).Build().Run();
//        }

//        public static IHostBuilder CreateHostBuilder(string[] args) =>
//            Host.CreateDefaultBuilder(args)
//                .ConfigureServices((hostContext, services) =>
//                {
//                    services.AddHostedService<Worker>();
//                });
//    }
//}
