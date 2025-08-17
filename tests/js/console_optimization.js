(function() {
    var originalLog = console.log;
    console.log = function() {
        var args = Array.prototype.slice.call(arguments);
        var processedArgs = args.map(function(arg) {
            if (Array.isArray(arg) && arg.length > 1000) {
                return '[Array of ' + arg.length + ' elements]';
            }
            return arg;
        });
        return originalLog.apply(console, processedArgs);
    };
})();