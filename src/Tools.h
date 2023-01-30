#ifndef INCLUDE_GUARD_TOOLS
#define INCLUDE_GUARD_TOOLS

#include <vector>
#include <iostream>
#include <gmpxx.h>

namespace tools
{
    // Stolen for now
    std::vector<uint64_t> generate_primetable(uint64_t prime_table_limit)
    {
        if (prime_table_limit < 2) return {};
        std::vector<uint64_t> compositeTable(prime_table_limit/128ULL + 1ULL, 0ULL); // Booleans indicating whether an odd number is composite: 0000100100101100...
        for (uint64_t f(3ULL) ; f*f <= prime_table_limit ; f += 2ULL) { // Eliminate f and its multiples m for odd f from 3 to square root of the limit
            if (compositeTable[f >> 7ULL] & (1ULL << ((f >> 1ULL) & 63ULL))) continue; // Skip if f is composite (f and its multiples were already eliminated)
            for (uint64_t m((f*f) >> 1ULL) ; m <= (prime_table_limit >> 1ULL) ; m += f) // Start eliminating at f^2 (multiples of f below were already eliminated)
                compositeTable[m >> 6ULL] |= 1ULL << (m & 63ULL);
        }
        std::vector<uint64_t> primeTable(1, 2);
        for (uint64_t i(1ULL) ; (i << 1ULL) + 1ULL <= prime_table_limit ; i++) { // Fill the prime table using the composite table
            if (!(compositeTable[i >> 6ULL] & (1ULL << (i & 63ULL))))
                primeTable.push_back((i << 1ULL) + 1ULL); // Add prime number 2i + 1
        }
        return primeTable;
    }

    mpz_class get_primorial(uint64_t primorial_number)
    {
        std::vector<uint64_t> primes = generate_primetable((primorial_number*primorial_number)+1);

        mpz_class primorial(1);

        for(int i = 0; i < primorial_number; i++)
        {
            mpz_mul_ui(primorial.get_mpz_t(), primorial.get_mpz_t(), primes[i]);
        }


        return primorial;
    }


    std::vector<uint64_t> get_primorial_inverses(const mpz_class& primorial, const std::vector<uint64_t>& v)
    {
        std::vector<uint64_t> inverses;

        for(const auto& i : v)
        {
            mpz_class modular_inverse;
            mpz_class prime(i);

            mpz_invert(modular_inverse.get_mpz_t(), primorial.get_mpz_t(), prime.get_mpz_t());

            inverses.push_back(modular_inverse.get_ui());
        }

        return inverses;
    }


    std::string get_difficulty_seed(uint32_t d)
    {

    }

    void save_tuples(const std::vector<mpz_class>& tuples, const std::string& tuple_file, uint64_t tuple_type)
    {

    }



};


#endif