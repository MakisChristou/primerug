#ifndef INCLUDE_GUARD_STATS
#define INCLUDE_GUARD_STATS

#include <vector>
#include <iostream>
#include <chrono>
#include <cmath>
#include <sstream>

using namespace std::chrono;
class Stats
{
    public:
        uint64_t _pattern_size;
        std::vector<uint64_t> _tuple_counts;
        std::chrono::time_point<std::chrono::system_clock> _duration;

    Stats(uint64_t pattern_size)
    {
        _pattern_size = pattern_size;
        _tuple_counts.resize(pattern_size, 0);
        _duration = std::chrono::system_clock::now();
    }

    uint64_t cps()
    {
        auto end = std::chrono::system_clock::now();
        std::chrono::duration<double> elapsed_seconds = end - _duration;

        uint64_t elapsed = elapsed_seconds.count();

        if(_tuple_counts.size() > 0 && elapsed > 0)
        {
            return _tuple_counts[0]/elapsed;
        }
        return 0;
    }
    
    double r()
    {
        if(_tuple_counts.size() > 0)
        {
            uint64_t single_tuples = _tuple_counts[0];
            uint64_t twin_tuples = _tuple_counts[1];

            double ratio = ((double)single_tuples)/(twin_tuples);

            return ratio;
        }
        return 0.0;
    }

    double get_eta()
    {
        double r = this->r();
        double tuple_length = this->_pattern_size;
        double cps = this->cps();

        if(r == 0.0 || cps == 0.0)
        {
            return 0.0;
        }
        else
        {
            return std::pow(r, tuple_length)/cps;
        }


    }
    
    std::string get_human_readable_stats()
    {
        std::ostringstream oss;
        oss << "c/s: " << this->cps() << " , r: " << this->r() << " ";

        for(auto& o : this->_tuple_counts)
            oss << o << ", ";
        
        oss << get_human_readable_eta();
        return oss.str();
    }

    std::string get_human_readable_eta()
    {
        std::ostringstream oss;
        oss << "eta: ";

        double eta_in_seconds = this->get_eta();
        
        if(eta_in_seconds < 60)
        {
            oss << eta_in_seconds << " s";
        }
        else if(eta_in_seconds < 3600)
        {
            oss << eta_in_seconds/60 << " min";
        }
        else if(eta_in_seconds < 86400)
        {
            oss << eta_in_seconds/3600 << " h";
        }
        else if(eta_in_seconds < 31556952)
        {
            oss << eta_in_seconds/86400 << " d";
        }
        else
        {
            oss << eta_in_seconds/31556952 << " y";
        }

        return oss.str();
    }

};


#endif