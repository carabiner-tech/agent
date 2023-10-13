from utils import lcm

def euler_problem_5(limit=20):
    """Solves Project Euler Problem 5 by calculating and printing the smallest positive number that is evenly divisible by all of the numbers from 1 to the specified limit."""
    print(lcm(*range(1, limit + 1)))

if __name__ == "__main__":
    euler_problem_5()