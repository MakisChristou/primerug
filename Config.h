#ifndef INCLUDE_GUARD_CONFIG
#define INCLUDE_GUARD_CONFIG

#include <vector>
#include <iostream>
#include <sstream>
#include <algorithm>

class Config
{
    public:
        uint32_t _d;
        std::vector<uint64_t> _constellation_pattern;
        uint64_t _m;
        uint64_t _o;
        uint64_t _prime_table_limit;

    Config(uint32_t d, std::string constellation_pattern, uint64_t m, uint64_t o, uint64_t prime_table_limit)
    {
        _d = d;
        _m = m;
        _o = o;
        _prime_table_limit = prime_table_limit;

        _constellation_pattern = get_pattern_vector(constellation_pattern);
    }

    std::vector<uint64_t> get_pattern_vector(std::string offsets)
    {
        offsets.erase(std::remove_if(offsets.begin(), offsets.end(), isspace), offsets.end());

        // Vector of string to save tokens
        std::vector <std::uint64_t> tokens;
        
        // stringstream class check1
        std::stringstream check1(offsets);
        std::string intermediate;
        
        // Tokenizing w.r.t. comma ','
        while(getline(check1, intermediate, ','))
            tokens.push_back(std::stoul(intermediate));
        
        return tokens;
    }

};


#endif