module TestAccount::Negation_main {
    fun neg_log(x: bool): bool {
        !x
    }

    spec neg_log {
        ensures result == !x;
    }
}
