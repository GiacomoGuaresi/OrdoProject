using Microsoft.Extensions.Configuration;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace OrdoProject
{
    public class OrdoService
    {
        public class DirectoryEntry
        {
            public DirectoryEntry(string path, List<string> match)
            {
                this.path = path;
                this.match = match;
            }

            public string path { get; set; }
            public List<string> match { get; set; }
        }

        private readonly IConfiguration _configuration;

        private List<DirectoryEntry> directoryEntries;
        private string directoryEntryDefault;
        private string InputDirectory;
        private string OutputDirectory;
        private List<string> IgnoredExtension;


        public OrdoService(IConfiguration configuration)
        {
            //https://docs.microsoft.com/it-it/dotnet/core/extensions/windows-service
            _configuration = configuration;
            directoryEntries = new List<DirectoryEntry>();

            InputDirectory = _configuration.GetValue<string>("InputDirectory");
            OutputDirectory = _configuration.GetValue<string>("OutputDirectory");
            foreach (IConfigurationSection FilteredFilesDirectory in _configuration.GetSection("FilteredFilesDirectory").Get<IConfigurationSection[]>())
            {
                string[] match = _configuration.GetSection(FilteredFilesDirectory.Path).Get<string[]>();
                directoryEntries.Add(new DirectoryEntry(FilteredFilesDirectory.Key, new List<string>(match)));
                Directory.CreateDirectory(Path.Combine(OutputDirectory, FilteredFilesDirectory.Key));
            }

            IgnoredExtension = new List<string>(_configuration.GetSection("IgnoredExtension").Get<string[]>());

            directoryEntryDefault = _configuration.GetValue<string>("OtherFileDirectory");
            Directory.CreateDirectory(Path.Combine(OutputDirectory, directoryEntryDefault));
        }

        public List<string> ExecuteAsync()
        {
            List<string> logs = new List<string>();

            string[] files = Directory.GetFiles(InputDirectory);

            foreach (string file in files)
            {
                string folder = GetDirectory(file);
                if (folder == "") //salta file ignorati 
                    continue;
                string filename = Path.GetFileName(file);
                string newPath = Path.Combine(OutputDirectory, folder, filename);
                if (File.Exists(newPath))
                    newPath = Path.Combine(OutputDirectory, folder, DateTime.Now.ToString("yyyyMMddHHmmss") + "_" + filename);
                File.Move(file, newPath);
                logs.Add("MOVED: " + filename + " TO " + Path.Combine(OutputDirectory, folder));
            }

            return logs; 
        }
        
        private string GetDirectory(string fileName)
        {
            string extension = Path.GetExtension(fileName).ToLower();

            if (IgnoredExtension.Contains(extension) == true)
                return "";

            DirectoryEntry foundedDir = directoryEntries.FirstOrDefault(m => m.match.Contains(extension) == true);
            if (foundedDir == null)
                return directoryEntryDefault;
            else
                return foundedDir.path;
        }
    }
}
