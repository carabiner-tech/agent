def euler_problem_9(sum=1000):
    """Solves Project Euler Problem 9 by finding and printing the product abc of the Pythagorean triplet for which a + b + c = 1000."""
    for a in range(1, sum):
        for b in range(a, sum - a):
            c = sum - a - b
            if a * a + b * b == c * c:
                print(a * b * c)
                return

if __name__ == "__main__":
    euler_problem_9()