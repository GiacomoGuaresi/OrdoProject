using Microsoft.Data.Sqlite;
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
        private string PortalDirectory;
        private string OutputDirectory;
        private List<string> IgnoredExtension;
        private int AutoCleaningSeconds;
        private DateTime LastAutoCleaningExecution;
        private bool CleaningEnabled; 
        private string IgnoreFileStartingWith;

        public OrdoService(IConfiguration configuration)
        {
            //https://docs.microsoft.com/it-it/dotnet/core/extensions/windows-service
            _configuration = configuration;
            directoryEntries = new List<DirectoryEntry>();

            InputDirectory = _configuration.GetValue<string>("InputDirectory");
            PortalDirectory = _configuration.GetValue<string>("PortalDirectory");
            OutputDirectory = _configuration.GetValue<string>("OutputDirectory");
            IgnoreFileStartingWith = _configuration.GetValue<string>("IgnoreFileStartingWith");

            CleaningEnabled = _configuration.GetValue<bool>("AutoCleaning:Enable");
            AutoCleaningSeconds = _configuration.GetValue<int>("AutoCleaning:Time");
            switch (_configuration.GetValue<string>("AutoCleaning:Scale"))
            {
                case "Minute":
                    AutoCleaningSeconds = AutoCleaningSeconds * 60;
                    break;
                case "Hours":
                    AutoCleaningSeconds = AutoCleaningSeconds * 60 * 60;
                    break;
                case "Day":
                    AutoCleaningSeconds = AutoCleaningSeconds * 60 * 60 * 24;
                    break;
                default:
                    break;
            }

            foreach (IConfigurationSection FilteredFilesDirectory in _configuration.GetSection("FilteredFilesDirectory").Get<IConfigurationSection[]>())
            {
                string[] match = _configuration.GetSection(FilteredFilesDirectory.Path).Get<string[]>();
                directoryEntries.Add(new DirectoryEntry(FilteredFilesDirectory.Key, new List<string>(match)));
                Directory.CreateDirectory(Path.Combine(OutputDirectory, FilteredFilesDirectory.Key));
            }

            IgnoredExtension = new List<string>(_configuration.GetSection("IgnoredExtension").Get<string[]>());

            directoryEntryDefault = _configuration.GetValue<string>("OtherFileDirectory");
            Directory.CreateDirectory(Path.Combine(OutputDirectory, directoryEntryDefault));

            using (var connection = new SqliteConnection("Data Source=OrdoDb.db"))
            {
                connection.Open();

                var command = connection.CreateCommand();
                command.CommandText = @"CREATE TABLE IF NOT EXISTS recordedFiles (fileName STRING PRIMARY KEY, foundIn DATETIME);";
                command.ExecuteReader();

            }

        }

        public List<string> ExecuteAsync()
        {
            List<string> logs = new List<string>();

            if (DateTime.Now >= LastAutoCleaningExecution.AddSeconds(AutoCleaningSeconds) && CleaningEnabled)
                logs = ExecuteCleaning(logs);

            //Pulizia Cartella portale
            RunClear(logs, PortalDirectory);

            return logs;
        }

        public List<string> ExecuteCleaning(List<string> logs)
        {
            //using (var connection = new SqliteConnection("Data Source=OrdoDb.db"))
            //{
            //    connection.Open();

            //    //Leggi tutti i file "vecchi"
            //    var command = connection.CreateCommand();
            //    command.CommandText = @"SELECT * FROM recordedFiles WHERE foundIn <= $foundIn";
            //    command.Parameters.AddWithValue("$foundIn", id);

            //    using (var reader = command.ExecuteReader())
            //    {
            //        while (reader.Read())
            //        {
            //            var name = reader.GetString(0);

            //            Console.WriteLine($"Hello, {name}!");
            //        }
            //    }
            //}

            logs.Add("AutoCleaning Started");
            LastAutoCleaningExecution = DateTime.Now;
            RunClear(logs, InputDirectory);
            RunClearDir(logs, InputDirectory);
            logs.Add("AutoCleaning Terminated");
            //Pulizia Cartella Input
            return logs; 
        }

        private List<string> RunClear(List<string> logs, string directory)
        {
            string[] files = Directory.GetFiles(directory);
            foreach (string file in files)
            {
                string filename = Path.GetFileName(file);
                string folder = GetDirectory(filename);
                if (folder == "") //salta file ignorati 
                    continue;
                string newPath = Path.Combine(OutputDirectory, folder, filename);
                if (File.Exists(newPath))
                    newPath = Path.Combine(OutputDirectory, folder, Path.GetFileNameWithoutExtension(filename) + " " + DateTime.Now.ToString("yyyy-MM-dd HH-mm-ss") + Path.GetExtension(filename));
                File.Move(file, newPath);
                logs.Add("MOVED: " + filename + " TO " + Path.Combine(OutputDirectory, folder));
            }

            return logs;
        }

        private List<string> RunClearDir(List<string> logs, string directory)
        {
            string[] dirs = Directory.GetDirectories(directory, @"*@*");
            foreach (string dir in dirs)
            {
                string dirname = Path.GetFileName(dir);
                string dirnameClear = dirname.Substring(0, dirname.LastIndexOf('@'));
                
                string folder = GetDirectoryDir(dir);
                if (folder == "") //salta file ignorati 
                    continue;
                
                string newPath = Path.Combine(OutputDirectory, folder, dirnameClear);
                
                if (Directory.Exists(newPath))
                    newPath = Path.Combine(OutputDirectory, folder, dirnameClear + " " + DateTime.Now.ToString("yyyy-MM-dd HH-mm-ss"));

                Directory.Move(dir, newPath);
                logs.Add("MOVED Directory: " + dirname + " TO " + Path.Combine(OutputDirectory, folder));
            }
            return logs; 
        }

        private string GetDirectory(string fileName)
        {
            string extension = Path.GetExtension(fileName).ToLower();

            //Check estensioni ignorate
            if (IgnoredExtension.Contains(extension) == true)
                return "";

            //Check file con nome ignorati
            if (fileName.StartsWith(IgnoreFileStartingWith))
                return "";
            
            DirectoryEntry foundedDir = directoryEntries.FirstOrDefault(m => m.match.Contains(extension) == true);
            if (foundedDir == null)
                return directoryEntryDefault;
            else
                return foundedDir.path;
        }

        private string GetDirectoryDir(string fileName)
        {
            string extension = "." + fileName.Substring(fileName.LastIndexOf('@') + 1);

            //Check estensioni ignorate
            if (IgnoredExtension.Contains(extension) == true)
                return "";

            DirectoryEntry foundedDir = directoryEntries.FirstOrDefault(m => m.match.Contains(extension) == true);
            if (foundedDir == null)
                return "";
            else
                return foundedDir.path;
        }
    }
}
