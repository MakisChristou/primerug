#include <iostream>
#include <chrono>
#include <vector>
#include <gmpxx.h>

#include "Stats.h"
#include "Tools.h"
#include "Config.h"

static const mpz_class mpz2(2);

inline bool fermat(const mpz_class& n)
{
	mpz_class r, nm1(n - 1);
	mpz_powm(r.get_mpz_t(), mpz2.get_mpz_t(), nm1.get_mpz_t(), n.get_mpz_t()); // r = 2^(n - 1) % n
	return r == 1;
}

inline uint8_t get_bit(std::vector<uint8_t>& sieve, uint64_t i)
{
    // Find which byte this bit is on
    uint64_t byte = i>>3;

    // Find the bit's position whithin the byte
    uint64_t position = i%8;

    // Shift left so value becomes LSB
    uint8_t shifted = sieve[byte]>>(8-position);

    // Ignore/Mask higher values
    uint8_t value = shifted & 0x1;

    return value;

}

inline void set_bit(std::vector<uint8_t>& sieve, uint64_t i, uint8_t value)
{
    // Find which byte this bit is on
    uint64_t byte = i>>3;

    // Find the bit's position whithin the byte
    uint64_t position = i%8;

    // Mask value's upper bits
    value = value & 0x1;  // e.g. 0x00000001

    // Align bit with position
    value = value << (8-position); // e.g 0x00010000

    // Set bit
    sieve[byte] |= value;
}

inline bool is_constellation(const mpz_class& n, const std::vector<uint64_t>& v, Stats& miner_stats)
{
    mpz_class candidate(n);

    int index = 0;

    miner_stats._tuple_counts[0]++;

    for(auto& offset : v)
    {
        // f = candidate + offset
        mpz_class f;
        mpz_add_ui(f.get_mpz_t(), candidate.get_mpz_t(), offset);

        if(!fermat(f))
        {
            return false;
        }

        // Update Tuple Stats
        // index+1 because we don't update the candidates
        miner_stats._tuple_counts[index+1]++;
        index++;
    }
    return true;
}

std::vector<uint8_t> get_eliminated_factors(const std::vector<uint64_t>& primes, const std::vector<uint64_t>& inverses,const mpz_class& t_prime, const mpz_class& offset, const std::vector<uint64_t>& v, uint64_t prime_table_limit)
{

    uint64_t k_max = 10;

    uint64_t sieve_size = prime_table_limit + prime_table_limit + k_max;

    std::vector<uint8_t> sieve((sieve_size)+1, 0);


    mpz_class t_prime_plus_offset;
    mpz_add(t_prime_plus_offset.get_mpz_t(), t_prime.get_mpz_t(), offset.get_mpz_t());

    std::cout << "Sieving" << std::endl;

    uint64_t i = 0;

    for(const auto& p : primes)
    {
        if(p!=0)
        {
            for(const auto& c_i : v)
            {

                mpz_class t_prime_plus_offset_plus_c_i;

                // (T2 + o + c_i)
                mpz_add_ui(t_prime_plus_offset_plus_c_i.get_mpz_t(), t_prime_plus_offset.get_mpz_t(), c_i);

                // ((T2 + o + c_i) % p)
                mpz_mod_ui(t_prime_plus_offset_plus_c_i.get_mpz_t(), t_prime_plus_offset_plus_c_i.get_mpz_t(), p);
                uint64_t r = t_prime_plus_offset_plus_c_i.get_ui();

                // f_p = ((p - ((T2 + o + c_i) % p))*p_m_inverse) % p
                uint64_t f_p = ((p-(r)) * inverses[i]) % p;

                set_bit(sieve, f_p, 1);

                // Sieve out multiples of f_p
                for(int k = 0; k<k_max; k++)
                {
                    f_p+=p;
                    set_bit(sieve, f_p, 1);
                }
            }
        }
        i++;
    }
    // Should we move?
    return sieve;
}

std::vector<uint8_t> get_eliminated_factors_working(const std::vector<uint64_t>& primes, const std::vector<uint64_t>& inverses,const mpz_class& t_prime, const mpz_class& offset, const std::vector<uint64_t>& v, uint64_t prime_table_limit)
{

    uint64_t k_max = 1000;

    uint64_t sieve_size = prime_table_limit + prime_table_limit + k_max;

    std::vector<uint8_t> sieve(sieve_size+1, 0);


    mpz_class t_prime_plus_offset;
    mpz_add(t_prime_plus_offset.get_mpz_t(), t_prime.get_mpz_t(), offset.get_mpz_t());

    std::cout << "Sieving" << std::endl;

    uint64_t i = 0;


    for(auto p : primes)
    {
        if(p != 0)
        {
            for(const auto& c_i : v)
            {
                mpz_class t_prime_plus_offset_plus_c_i;

                // (T2 + o + c_i)
                mpz_add_ui(t_prime_plus_offset_plus_c_i.get_mpz_t(), t_prime_plus_offset.get_mpz_t(), c_i);

                // ((T2 + o + c_i) % p)
                mpz_mod_ui(t_prime_plus_offset_plus_c_i.get_mpz_t(), t_prime_plus_offset_plus_c_i.get_mpz_t(), p);
                uint64_t r = t_prime_plus_offset_plus_c_i.get_ui();

                // f_p = ((p - ((T2 + o + c_i) % p))*p_m_inverse) % p
                uint64_t f_p = ((p-(r)) * inverses[i]) % p;

                // std::cout << " " <<  i  << " " << p << std::endl;
                // set_bit(sieve, f_p, 1);
                sieve[f_p] = 1;

                // Sieve out multiples of f_p
                // for(int k = 0; k<k_max; k++)
                // {
                //     // f_p+=p;
                //     // set_bit(sieve, f_p, 1);
                //     // sieve[f_p] = 1;
                // }
            }
        }
        i++;
    }

    // Should we move?
    return sieve;
}

void wheel_factorization(const std::vector<uint64_t>& v, mpz_class& t, const mpz_class& primorial, const mpz_class& offset, const std::vector<uint64_t>& primes, const std::vector<uint64_t>& inverses, uint64_t prime_table_limit)
{
    int primes_count = 0;
    int primality_tests = 0;


    mpz_class t_prime;
    mpz_class ret;

    // T2 = T + p_m - (T % p_m)
    mpz_add(t_prime.get_mpz_t(), t.get_mpz_t(), primorial.get_mpz_t());
    mpz_mod(ret.get_mpz_t(), t.get_mpz_t(), primorial.get_mpz_t());
    mpz_sub(t_prime.get_mpz_t(), t_prime.get_mpz_t(), ret.get_mpz_t());

    if(primorial >= t)
    {
        std::cout << "Pick Smaller primorial number" << std::endl;
        return;
    }

    // std::cout << "Candidates of the form: " << primorial << " * f + " << offset << " + " << t_prime << std::endl;
    std::cout << "Candidates of the form" << std::endl;

    std::vector<uint8_t> sieve = get_eliminated_factors_working(primes, inverses, t_prime, offset, v, prime_table_limit);

    std::cout << "sieve.size() = " << sieve.size() << std::endl;
    std::cout << "Sieve Size: " << sieve.size()/1000000 << " MB " << std::endl;

    uint64_t f_max = sieve.size();

    std::cout << "Primality Testing" << std::endl;

    std::cout << "v.size(): " << v.size() << std::endl;

    auto miner_stats = Stats{v.size()};

    mpz_class t_prime_plus_offset;

    mpz_add(t_prime_plus_offset.get_mpz_t(), t_prime.get_mpz_t(), offset.get_mpz_t());

    std::vector<mpz_class> tuples;

    uint64_t i = 0;

    for(const auto& bit : sieve)
    {
        // uint8_t eliminated = get_bit(sieve, i);
        uint8_t eliminated = bit;

        if((i % 100000) == 0)
        {   
            std::cout << miner_stats.get_human_readable_stats() << std::endl;
        }

        if(eliminated != 1)
        {
            mpz_class j;

            // j = p_m * f + o + T2
            mpz_mul_ui(j.get_mpz_t(), primorial.get_mpz_t(), i);
            mpz_add(j.get_mpz_t(), j.get_mpz_t(), t_prime_plus_offset.get_mpz_t());

            // Fermat Test on j
            if(is_constellation(j, v, miner_stats))
            {
                primes_count+=1;

                tuples.push_back(j);
                
                // Save them as we go, just in case
                tools::save_tuples(tuples, "tuples.txt", v.size());
            }
            primality_tests+=1;
        }
        i+=1;
    }

    std::cout << "Found " << primes_count << " tuples, with " << primality_tests << " primality tests, eliminated " << f_max - primality_tests << std::endl; 
}

void bruteforce(const std::vector<uint64_t>& v, mpz_class& t, const mpz_class& primorial, const mpz_class& offset, const std::vector<uint64_t>& primes, const std::vector<uint64_t>& inverses, uint64_t prime_table_limit)
{   
    mpz_class t_prime;
    mpz_class ret;

    // T2 = T + p_m - (T % p_m)
    mpz_add(t_prime.get_mpz_t(), t.get_mpz_t(), primorial.get_mpz_t());
    mpz_mod(ret.get_mpz_t(), t.get_mpz_t(), primorial.get_mpz_t());
    mpz_sub(t_prime.get_mpz_t(), t_prime.get_mpz_t(), ret.get_mpz_t());
    
    auto miner_stats = Stats{v.size()};

    mpz_class t_prime_plus_offset;
    mpz_add(t_prime_plus_offset.get_mpz_t(), t_prime.get_mpz_t(), offset.get_mpz_t());

    mpz_class j(2);

    for(int i = 0; i < 1000000; i++)
    {
        // mpz_class j;

        // j = p_m * f + o + T2
        mpz_mul_ui(j.get_mpz_t(), primorial.get_mpz_t(), i);
        mpz_add(j.get_mpz_t(), j.get_mpz_t(), t_prime_plus_offset.get_mpz_t());

        // Fermat Test on j
        if(is_constellation(j, v, miner_stats))
        {
            std::cout << "IS CONSTELLATION\n";
        }

        mpz_add_ui(j.get_mpz_t(), j.get_mpz_t(), 1);

    }
}

int main(int argc, char** argv)
{                        
    uint64_t prime_table_limit = 10000000;
    uint64_t m = 58;
    uint64_t offset = 600598127;
    std::vector<uint64_t> constellation_pattern{0, 2, 6, 8, 12, 18, 20, 26};
    uint64_t d = 100;

    Config config = Config(d, "0, 2, 6, 8, 12, 18, 20, 26", m, offset, prime_table_limit);

    mpz_class p_m = tools::get_primorial(m);

    std::vector<uint64_t> primes = tools::generate_primetable(prime_table_limit);

    std::vector<uint64_t> inverses = tools::get_primorial_inverses(p_m, primes);

    std::string t_str = "35382514261572877775443718275127654368455742689370898970579333127222089440767840029778961849392487105286882749492874572975965526331001174046077835955";

    t_str = tools::get_difficulty_seed(150);

    std::cout << "Difficulty Seed: " << t_str << std::endl;

    mpz_class t(t_str);

    wheel_factorization(constellation_pattern, t, p_m, offset, primes, inverses, prime_table_limit);
}