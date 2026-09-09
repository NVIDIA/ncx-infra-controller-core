// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package manager

import (
	"context"
	"fmt"
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/require"

	dbquery "github.com/NVIDIA/infra-controller/rest-api/flow/internal/db/query"
	inventorystore "github.com/NVIDIA/infra-controller/rest-api/flow/internal/inventory/store"
	"github.com/NVIDIA/infra-controller/rest-api/flow/internal/operation"
	taskcommon "github.com/NVIDIA/infra-controller/rest-api/flow/internal/task/common"
	"github.com/NVIDIA/infra-controller/rest-api/flow/internal/task/conflict"
	"github.com/NVIDIA/infra-controller/rest-api/flow/internal/task/operationrules"
	"github.com/NVIDIA/infra-controller/rest-api/flow/internal/task/operations"
	taskdef "github.com/NVIDIA/infra-controller/rest-api/flow/internal/task/task"
	identifier "github.com/NVIDIA/infra-controller/rest-api/flow/pkg/common/Identifier"
	"github.com/NVIDIA/infra-controller/rest-api/flow/pkg/common/devicetypes"
	"github.com/NVIDIA/infra-controller/rest-api/flow/pkg/inventoryobjects/rack"
)

type submitTaskInventory struct {
	inventorystore.Store
	rack *rack.Rack
}

func (s *submitTaskInventory) GetRackByIdentifier(
	_ context.Context,
	_ identifier.Identifier,
	_ bool,
) (*rack.Rack, error) {
	return s.rack, nil
}

func TestManagerImpl_SubmitTask(t *testing.T) {
	rackID := uuid.New()
	resolvedRack := newTestRack(rackID, "rack-1")
	unlinkedID := uuid.New()
	unlinked := newTestComponent(
		unlinkedID,
		rackID,
		devicetypes.ComponentTypeCompute,
		"compute-1",
	)
	unlinked.ComponentID = ""
	resolvedRack.AddComponent(unlinked)

	store := &managerTaskStore{}
	manager := &ManagerImpl{
		inventoryStore: &submitTaskInventory{rack: resolvedRack},
		taskStore:      store,
	}

	_, err := manager.SubmitTask(context.Background(), &operation.Request{
		Operation: testPowerControlOperation(t),
		TargetSpec: operation.TargetSpec{
			Racks: []operation.RackTarget{{
				Identifier: identifier.Identifier{ID: rackID},
			}},
		},
	})

	require.ErrorContains(t, err, "selected components not linked to actual inventory (1)")
	require.ErrorContains(t, err, unlinkedID.String())
	require.Zero(t, store.createTaskCalls)
}

func TestManagerImpl_ExecuteTask(t *testing.T) {
	rackID := uuid.New()
	resolvedRack := newTestRack(rackID, "rack-1")
	unlinked := newTestComponent(
		uuid.New(),
		rackID,
		devicetypes.ComponentTypeCompute,
		"compute-1",
	)
	unlinked.ComponentID = ""
	resolvedRack.AddComponent(unlinked)

	manager := &ManagerImpl{}
	resp, err := manager.executeTask(
		context.Background(),
		&taskdef.Task{
			ID:        uuid.New(),
			RackID:    rackID,
			Operation: testPowerControlOperation(t),
		},
		resolvedRack,
		nil,
	)

	require.Nil(t, resp)
	require.ErrorContains(t, err, "selected components not linked to actual inventory (1)")
}

func TestValidateResolvedRackTargets(t *testing.T) {
	tests := []struct {
		name         string
		op           operation.Wrapper
		componentIDs []string
		wantError    string
	}{
		{
			name: "linked components allow a disruptive operation",
			op: operation.Wrapper{
				Type: taskcommon.TaskTypePowerControl,
				Code: taskcommon.OpCodePowerControlPowerOff,
			},
			componentIDs: []string{"machine-1", "machine-2"},
		},
		{
			name: "unlinked component rejects a disruptive operation",
			op: operation.Wrapper{
				Type: taskcommon.TaskTypeFirmwareControl,
				Code: taskcommon.OpCodeFirmwareControlUpgrade,
			},
			componentIDs: []string{"machine-1", ""},
			wantError:    "selected components not linked to actual inventory (1)",
		},
		{
			name: "ingestion allows an unlinked expected component",
			op: operation.Wrapper{
				Type: taskcommon.TaskTypeBringUp,
				Code: taskcommon.OpCodeIngest,
			},
			componentIDs: []string{""},
		},
		{
			name: "inject expectation allows an unlinked expected component",
			op: operation.Wrapper{
				Type: taskcommon.TaskTypeInjectExpectation,
				Code: taskcommon.OpCodeInjectExpectation,
			},
			componentIDs: []string{""},
		},
		{
			name: "empty selected scope rejects every operation",
			op: operation.Wrapper{
				Type: taskcommon.TaskTypeBringUp,
				Code: taskcommon.OpCodeIngest,
			},
			wantError: "racks have no selected components",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			rackID := uuid.New()
			resolvedRack := newTestRack(rackID, "rack-1")
			componentFlowIDs := make([]uuid.UUID, 0, len(test.componentIDs))
			for i, externalID := range test.componentIDs {
				flowID := uuid.New()
				componentFlowIDs = append(componentFlowIDs, flowID)
				comp := newTestComponent(
					flowID,
					rackID,
					devicetypes.ComponentTypeCompute,
					fmt.Sprintf("compute-%d", i),
				)
				comp.ComponentID = externalID
				resolvedRack.AddComponent(comp)
			}

			err := validateResolvedRackTargets(test.op, map[uuid.UUID]*rack.Rack{
				rackID: resolvedRack,
			})
			if test.wantError != "" {
				require.ErrorContains(t, err, test.wantError)
				for i, externalID := range test.componentIDs {
					if externalID == "" {
						require.ErrorContains(t, err, componentFlowIDs[i].String())
					}
				}
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestCreateAndExecuteTaskReturnsExistingIdempotentTaskBeforeRackConflict(t *testing.T) {
	ctx := context.Background()
	rackID := uuid.New()
	componentID := uuid.New()
	taskID := uuid.New()
	idempotencyKey := "operation-run-target:" + uuid.NewString()
	op := testPowerControlOperation(t)
	targetRack := newTestRack(rackID, "rack-1")
	targetRack.AddComponent(newTestComponent(
		componentID,
		rackID,
		devicetypes.ComponentTypeCompute,
		"compute-1",
	))
	existingTask := &taskdef.Task{
		ID:             taskID,
		Operation:      op,
		RackID:         rackID,
		Status:         taskcommon.TaskStatusPending,
		ExecutionID:    `{"workflow_id":"workflow","run_id":"run"}`,
		IdempotencyKey: idempotencyKey,
		Attributes: taskcommon.TaskAttributes{
			ComponentsByType: map[devicetypes.ComponentType][]uuid.UUID{
				devicetypes.ComponentTypeCompute: {componentID},
			},
		},
	}
	store := &managerTaskStore{
		activeTasksByRack: map[uuid.UUID][]*taskdef.Task{
			rackID: {
				{
					ID:        uuid.New(),
					Operation: op,
					RackID:    rackID,
					Status:    taskcommon.TaskStatusRunning,
					Attributes: taskcommon.TaskAttributes{
						ComponentsByType: map[devicetypes.ComponentType][]uuid.UUID{
							devicetypes.ComponentTypeCompute: {componentID},
						},
					},
				},
			},
		},
		taskByIdempotencyKey: map[string]*taskdef.Task{
			idempotencyKey: existingTask,
		},
	}
	manager := &ManagerImpl{
		taskStore:           store,
		conflictResolver:    conflict.NewResolver(store),
		maxWaitingPerRack:   defaultMaxWaitingPerRack,
		defaultQueueTimeout: defaultQueueTimeout,
	}

	gotTaskID, err := manager.createAndExecuteTask(ctx, &operation.Request{
		Operation:        op,
		Description:      "retry operation-run target",
		ConflictStrategy: operation.ConflictStrategyReject,
		RequiredRackID:   rackID,
		IdempotencyKey:   idempotencyKey,
	}, targetRack)

	require.NoError(t, err)
	require.Equal(t, taskID, gotTaskID)
	require.Zero(t, store.listActiveCalls)
	require.Zero(t, store.createTaskCalls)
}

func TestCreateAndExecuteTaskSchedulesExistingIdempotentTaskWithoutExecutionID(t *testing.T) {
	ctx := context.Background()
	rackID := uuid.New()
	componentID := uuid.New()
	taskID := uuid.New()
	idempotencyKey := "operation-run-target:" + uuid.NewString()
	op := testPowerControlOperation(t)
	targetRack := newTestRack(rackID, "rack-1")
	targetRack.AddComponent(newTestComponent(
		componentID,
		rackID,
		devicetypes.ComponentTypeCompute,
		"compute-1",
	))
	existingTask := &taskdef.Task{
		ID:             taskID,
		Operation:      op,
		RackID:         rackID,
		Status:         taskcommon.TaskStatusPending,
		IdempotencyKey: idempotencyKey,
		Attributes: taskcommon.TaskAttributes{
			ComponentsByType: map[devicetypes.ComponentType][]uuid.UUID{
				devicetypes.ComponentTypeCompute: {componentID},
			},
		},
	}
	store := &managerTaskStore{
		taskByIdempotencyKey: map[string]*taskdef.Task{
			idempotencyKey: existingTask,
		},
	}
	executor := &managerExecutor{executionID: `{"workflow_id":"workflow","run_id":"run"}`}
	manager := &ManagerImpl{
		taskStore:           store,
		executor:            executor,
		maxWaitingPerRack:   defaultMaxWaitingPerRack,
		defaultQueueTimeout: defaultQueueTimeout,
	}

	gotTaskID, err := manager.createAndExecuteTask(ctx, &operation.Request{
		Operation:        op,
		Description:      "retry operation-run target",
		ConflictStrategy: operation.ConflictStrategyReject,
		RequiredRackID:   rackID,
		IdempotencyKey:   idempotencyKey,
	}, targetRack)

	require.NoError(t, err)
	require.Equal(t, taskID, gotTaskID)
	require.Equal(t, 1, executor.executeCalls)
	require.Equal(t, taskID, executor.lastRequest.Info.TaskID)
	require.Equal(t, 1, store.updateScheduledCalls)
	require.Equal(t, executor.executionID, store.updatedScheduledTask.ExecutionID)
	require.Zero(t, store.listActiveCalls)
	require.Zero(t, store.createTaskCalls)
}

func testPowerControlOperation(t *testing.T) operation.Wrapper {
	t.Helper()

	info, err := (&operations.PowerControlTaskInfo{
		Operation: operations.PowerOperationPowerOn,
	}).Marshal()
	require.NoError(t, err)

	return operation.Wrapper{
		Type: taskcommon.TaskTypePowerControl,
		Code: taskcommon.OpCodePowerControlPowerOn,
		Info: info,
	}
}

type managerTaskStore struct {
	activeTasksByRack    map[uuid.UUID][]*taskdef.Task
	taskByIdempotencyKey map[string]*taskdef.Task
	listActiveCalls      int
	createTaskCalls      int
	lockKeyCalls         int
	lockRackCalls        int
	updateScheduledCalls int
	updatedScheduledTask *taskdef.Task
}

func (s *managerTaskStore) RunInTransaction(
	ctx context.Context,
	fn func(context.Context) error,
) error {
	return fn(ctx)
}

func (s *managerTaskStore) CreateTask(_ context.Context, _ *taskdef.Task) error {
	s.createTaskCalls++
	return fmt.Errorf("CreateTask should not be called")
}

func (s *managerTaskStore) LockRack(_ context.Context, _ uuid.UUID) error {
	s.lockRackCalls++
	return nil
}

func (s *managerTaskStore) LockIdempotencyKey(_ context.Context, _ string) error {
	s.lockKeyCalls++
	return nil
}

func (s *managerTaskStore) GetTaskByIdempotencyKey(
	_ context.Context,
	key string,
) (*taskdef.Task, error) {
	return s.taskByIdempotencyKey[key], nil
}

func (s *managerTaskStore) GetTask(_ context.Context, _ uuid.UUID) (*taskdef.Task, error) {
	panic("managerTaskStore.GetTask: not implemented")
}

func (s *managerTaskStore) GetTasks(_ context.Context, _ []uuid.UUID) ([]*taskdef.Task, error) {
	panic("managerTaskStore.GetTasks: not implemented")
}

func (s *managerTaskStore) ListTasks(
	_ context.Context,
	_ *taskcommon.TaskListOptions,
	_ *dbquery.Pagination,
) ([]*taskdef.Task, int32, error) {
	panic("managerTaskStore.ListTasks: not implemented")
}

func (s *managerTaskStore) ListNonTerminalTasksForRacks(
	_ context.Context,
	_ []uuid.UUID,
) ([]*taskdef.Task, error) {
	panic("managerTaskStore.ListNonTerminalTasksForRacks: not implemented")
}

func (s *managerTaskStore) UpdateScheduledTask(_ context.Context, task *taskdef.Task) error {
	s.updateScheduledCalls++
	s.updatedScheduledTask = task
	return nil
}

func (s *managerTaskStore) UpdateTaskStatus(
	_ context.Context,
	_ *taskdef.TaskStatusUpdate,
) error {
	panic("managerTaskStore.UpdateTaskStatus: not implemented")
}

func (s *managerTaskStore) UpdateTaskReport(
	_ context.Context,
	_ *taskdef.TaskReportUpdate,
) error {
	panic("managerTaskStore.UpdateTaskReport: not implemented")
}

func (s *managerTaskStore) ListActiveTasksForRack(
	_ context.Context,
	rackID uuid.UUID,
) ([]*taskdef.Task, error) {
	s.listActiveCalls++
	return s.activeTasksByRack[rackID], nil
}

func (s *managerTaskStore) ListWaitingTasksForRack(
	_ context.Context,
	_ uuid.UUID,
) ([]*taskdef.Task, error) {
	panic("managerTaskStore.ListWaitingTasksForRack: not implemented")
}

func (s *managerTaskStore) CountWaitingTasksForRack(_ context.Context, _ uuid.UUID) (int, error) {
	panic("managerTaskStore.CountWaitingTasksForRack: not implemented")
}

func (s *managerTaskStore) ListRacksWithWaitingTasks(_ context.Context) ([]uuid.UUID, error) {
	panic("managerTaskStore.ListRacksWithWaitingTasks: not implemented")
}

func (s *managerTaskStore) CreateRule(
	_ context.Context,
	_ *operationrules.OperationRule,
) error {
	panic("managerTaskStore.CreateRule: not implemented")
}

func (s *managerTaskStore) UpdateRule(
	_ context.Context,
	_ uuid.UUID,
	_ map[string]interface{},
) error {
	panic("managerTaskStore.UpdateRule: not implemented")
}

func (s *managerTaskStore) DeleteRule(_ context.Context, _ uuid.UUID) error {
	panic("managerTaskStore.DeleteRule: not implemented")
}

func (s *managerTaskStore) SetRuleAsDefault(_ context.Context, _ uuid.UUID) error {
	panic("managerTaskStore.SetRuleAsDefault: not implemented")
}

func (s *managerTaskStore) GetRule(
	_ context.Context,
	_ uuid.UUID,
) (*operationrules.OperationRule, error) {
	panic("managerTaskStore.GetRule: not implemented")
}

func (s *managerTaskStore) GetRuleByName(
	_ context.Context,
	_ string,
) (*operationrules.OperationRule, error) {
	panic("managerTaskStore.GetRuleByName: not implemented")
}

func (s *managerTaskStore) GetDefaultRule(
	_ context.Context,
	_ taskcommon.TaskType,
	_ string,
) (*operationrules.OperationRule, error) {
	panic("managerTaskStore.GetDefaultRule: not implemented")
}

func (s *managerTaskStore) GetRuleByOperationAndRack(
	_ context.Context,
	_ taskcommon.TaskType,
	_ string,
	_ *uuid.UUID,
) (*operationrules.OperationRule, error) {
	panic("managerTaskStore.GetRuleByOperationAndRack: not implemented")
}

func (s *managerTaskStore) ListRules(
	_ context.Context,
	_ *taskcommon.OperationRuleListOptions,
	_ *dbquery.Pagination,
) ([]*operationrules.OperationRule, int32, error) {
	panic("managerTaskStore.ListRules: not implemented")
}

func (s *managerTaskStore) AssociateRuleWithRack(
	_ context.Context,
	_ uuid.UUID,
	_ uuid.UUID,
) error {
	panic("managerTaskStore.AssociateRuleWithRack: not implemented")
}

func (s *managerTaskStore) DisassociateRuleFromRack(
	_ context.Context,
	_ uuid.UUID,
	_ taskcommon.TaskType,
	_ string,
) error {
	panic("managerTaskStore.DisassociateRuleFromRack: not implemented")
}

func (s *managerTaskStore) GetRackRuleAssociation(
	_ context.Context,
	_ uuid.UUID,
	_ taskcommon.TaskType,
	_ string,
) (*uuid.UUID, error) {
	panic("managerTaskStore.GetRackRuleAssociation: not implemented")
}

func (s *managerTaskStore) ListRackRuleAssociations(
	_ context.Context,
	_ uuid.UUID,
) ([]*operationrules.RackRuleAssociation, error) {
	panic("managerTaskStore.ListRackRuleAssociations: not implemented")
}

var _ interface {
	RunInTransaction(context.Context, func(context.Context) error) error
} = (*managerTaskStore)(nil)

type managerExecutor struct {
	executionID  string
	executeCalls int
	lastRequest  *taskdef.ExecutionRequest
}

func (e *managerExecutor) Start(context.Context) error {
	return nil
}

func (e *managerExecutor) Stop(context.Context) error {
	return nil
}

func (e *managerExecutor) Type() taskcommon.ExecutorType {
	return taskcommon.ExecutorTypeTemporal
}

func (e *managerExecutor) Execute(
	_ context.Context,
	req *taskdef.ExecutionRequest,
) (*taskdef.ExecutionResponse, error) {
	e.executeCalls++
	e.lastRequest = req
	return &taskdef.ExecutionResponse{ExecutionID: e.executionID}, nil
}

func (e *managerExecutor) CheckStatus(
	context.Context,
	string,
) (taskcommon.TaskStatus, error) {
	panic("managerExecutor.CheckStatus: not implemented")
}

func (e *managerExecutor) TerminateTask(context.Context, string, string) error {
	panic("managerExecutor.TerminateTask: not implemented")
}
