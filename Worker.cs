using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Configuration.Json;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace OrdoProject
{
    public class Worker : BackgroundService
    {
        private readonly ILogger<Worker> _logger;
        private readonly IConfiguration _configuration;
        private readonly OrdoService _ordoService;

        public Worker(ILogger<Worker> logger, IConfiguration configuration, OrdoService ordoService)
        {
            _logger = logger;
            _configuration = configuration;
            _ordoService = ordoService; 
        }

        protected override async Task ExecuteAsync(CancellationToken stoppingToken)
        {
            while (!stoppingToken.IsCancellationRequested)
            {
                List<string> logs = _ordoService.ExecuteAsync();
                logs.ForEach(m => _logger.LogInformation(m));
                
                await Task.Delay(TimeSpan.FromSeconds(1), stoppingToken);
            }
        }

    }
}
