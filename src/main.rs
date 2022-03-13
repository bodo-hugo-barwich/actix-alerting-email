/*
* @author Bodo (Hugo) Barwich
* @version 2022-03-13
* @package Grafana Alerting
* @subpackage Email Micro Service

* This Module runs the `main()` function in the Crate "alerting_email"
*
*---------------------------------
* Requirements:
* - The Rust Crate "alerting_email" must be installed
*/

fn main() -> std::io::Result<()> {
    alerting_email::main()
}
