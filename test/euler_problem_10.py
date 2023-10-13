from utils import generate_primes

def euler_problem_10(limit=2000000):
    """Solves Project Euler Problem 10 by calculating and printing the sum of all the primes below the specified limit."""
    print(sum(generate_primes(limit)))

if __name__ == "__main__":
    euler_problem_10()