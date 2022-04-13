using System;
using System.Collections.Generic;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using OrdoProject;

namespace OrdoCleaning
{
    class Program
    {
        static IConfiguration configuration;
        static IConfiguration configurationService;
        static ILogger logger; 

        static void Main(string[] args)
        {
            configuration = new ConfigurationBuilder()
                .AddJsonFile("appsettings.json")
                .AddEnvironmentVariables()
                .Build();

            using var loggerFactory = LoggerFactory.Create(builder => builder.AddConfiguration(configuration));
            logger = loggerFactory.CreateLogger<Program>();

            string ServiceAppsettings = configuration.GetValue<string>("ServiceAppsettings");

            configurationService = new ConfigurationBuilder()
                .AddJsonFile(ServiceAppsettings)
                .AddEnvironmentVariables()
                .Build();

            Run();
        }
        
        static void Run()
        {
            OrdoService ordoService = new OrdoService(configurationService);
            List<string> logs = ordoService.ExecuteCleaning(new List<string>());
            logs.ForEach(m => logger.LogInformation(m));
        }
    }
}
