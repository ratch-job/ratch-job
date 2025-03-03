#!/bin/sh

set -o errexit

action=$1

usage() {
    echo "cmd args unvalid"
    echo "usage: $0 start | start_debug | restart | restart_debug  | kill | clean | test_naming"
    exit 2
}

app_name='ratchjob'

test_dir='cluster_example'

#defuat path
app_path="./target/release/$app_name"

kill() {
    echo "Killing all running ${app_name}"
    if [ "$(uname)" = "Darwin" ]; then
        if pgrep -xq -- "${app_name}"; then
            pkill -f "${app_name}"
        fi
    else
        set +e # killall will error if finds no process to kill
        killall ${app_name}
        set -e
    fi
}

#kill
#sleep 1

clean_cluster_dir() {
    echo "init cluster dir: $test_dir"
    rm -rf $test_dir
    mkdir -p $test_dir
}

start_cluster() {
    echo "start node:1"
    local env_file="$test_dir/env_01"
    cat >$env_file <<EOF
#file:env01
#RUST_LOG=debug|info|warn|error, default is info
RATCH_HTTP_API_PORT=8725
RATCH_HTTP_CONSOLE_PORT=8825
RATCH_GRPC_CLUSTER_PORT=8925
RATCH_DATA_DIR=cluster_example/db_01
RATCH_RAFT_NODE_ID=1
RATCH_RAFT_NODE_ADDR=127.0.0.1:8925
RATCH_RAFT_AUTO_INIT=true
EOF
    nohup ${app_path} -e $env_file >"$test_dir/node_01.log" &
    sleep 1

    echo "start node:2"
    local env_file="$test_dir/env_02"
    cat >$env_file <<EOF
#file:env02
#RUST_LOG=debug|info|warn|error, default is info
RATCH_HTTP_API_PORT=8726
RATCH_HTTP_CONSOLE_PORT=8826
RATCH_GRPC_CLUSTER_PORT=8926
RATCH_DATA_DIR=cluster_example/db_02
RATCH_RAFT_NODE_ID=2
RATCH_RAFT_NODE_ADDR=127.0.0.1:8926
RATCH_RAFT_JOIN_ADDR=127.0.0.1:8925
EOF
    nohup ${app_path} -e $env_file >"$test_dir/node_02.log" &
    sleep 1

    echo "start node:3"
    local env_file="$test_dir/env_03"
    cat >$env_file <<EOF
#file:env03
#RUST_LOG=debug|info|warn|error, default is info
RATCH_HTTP_API_PORT=8727
RATCH_HTTP_CONSOLE_PORT=8827
RATCH_GRPC_CLUSTER_PORT=8927
RATCH_DATA_DIR=cluster_example/db_03
RATCH_RAFT_NODE_ID=3
RATCH_RAFT_NODE_ADDR=127.0.0.1:8927
RATCH_RAFT_JOIN_ADDR=127.0.0.1:8925
EOF
    nohup ${app_path} -e $env_file >"$test_dir/node_03.log" &
    sleep 1
}

query_node_metrics() {
    echo "\n the node1 raft metrics"
    curl "http://127.0.0.1:8725/api/v1/raft/metrics"

    echo "\n the node2 raft metrics"
    curl "http://127.0.0.1:8726/api/v1/raft/metrics"

    echo "\n the node3 raft metrics"
    curl "http://127.0.0.1:8727/api/v1/raft/metrics"
}

#start_cluster

#query_node_metrics

test_add_job_to_cluster() {
  echo "\nset job info to node 1"
  curl -X POST "http://127.0.0.1:8725/api/v1/job/create" -H 'Content-Type: application/json' -d '{"appName":"xxl-job-executor-sample","namespace":"xxl","handleName":"demoJobHandler","scheduleType":"CRON","cronValue":"0/15 * * * * *","blockingStrategy":"COVER_EARLY"}'
  sleep 1

  echo "\nset job info to node 2"
  curl -X POST "http://127.0.0.1:8726/api/v1/job/create" -H 'Content-Type: application/json' -d '{"appName":"job-executor-sample","namespace":"xxl","handleName":"demoJobHandler02","scheduleType":"CRON","cronValue":"0/15 * * * * *","blockingStrategy":"COVER_EARLY"}'
  sleep 1

  echo "\nset job info to node 3"
  curl -X POST "http://127.0.0.1:8727/api/v1/job/create" -H 'Content-Type: application/json' -d '{"appName":"job-executor-sample","namespace":"xxl","handleName":"demoJobHandler03","scheduleType":"CRON","cronValue":"0/15 * * * * *","blockingStrategy":"COVER_EARLY"}'
  sleep 1

  echo "\nquery job info from node 1"
  curl "http://127.0.0.1:8725/api/v1/job/list?app_name=job-executor-sample"

  echo "\nquery job info from node 2"
  curl "http://127.0.0.1:8726/api/v1/job/list?app_name=job-executor-sample"

  echo "\nquery job info from node 3"
  curl "http://127.0.0.1:8727/api/v1/job/list?app_name=job-executor-sample"


}

#query_node_metrics

restart_cluster() {

    kill

    echo "\nrestart cluster"

    sleep 1

    start_cluster

    sleep 1

    echo "\ncluster restart metrics:"

    query_node_metrics

    echo "\n\nwait until the clusters restart (need 5 seconds)"

    # the async-raft-ext all clusters restart need 5 seconds
    sleep 6
    query_node_metrics
}

#restart_cluster

#kill

start() {
    kill
    sleep 1
    cargo build --release
    app_path="./target/release/$app_name"
    clean_cluster_dir
    start_cluster
    query_node_metrics
    test_add_job_to_cluster
    query_node_metrics
}

start_debug() {
    kill
    sleep 1
    cargo build
    app_path="./target/debug/$app_name"
    clean_cluster_dir
    start_cluster
    query_node_metrics
    test_add_job_to_cluster
    query_node_metrics
}

restart() {
    cargo build --release
    app_path="./target/release/$app_name"
    restart_cluster
    test_add_job_to_cluster
    query_node_metrics
}

restart_debug() {
    cargo build
    app_path="./target/debug/$app_name"
    restart_cluster
    test_add_job_to_cluster
    query_node_metrics
}

main() {
    case $action in
    start)
        start
        ;;
    start_debug)
        start_debug
        ;;
    restart)
        restart
        ;;
    restart_debug)
        restart_debug
        ;;
    clean)
        kill
        sleep 1
        clean_cluster_dir
        ;;
    test_naming)
        test_naming_cluster
        ;;
    kill)
        kill
        ;;
    *)
        usage
        ;;
    esac
}
main
echo "\n==== end ===="
