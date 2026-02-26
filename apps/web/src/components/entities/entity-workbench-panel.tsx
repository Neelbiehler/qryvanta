"use client";

import { Notice } from "@qryvanta/ui";

import {
  RuntimeOperationsSection,
  SchemaDesignSection,
  WorkbenchOverview,
} from "@/components/entities/entity-workbench/sections";
import { useEntityWorkbenchPanel } from "@/components/entities/entity-workbench/use-entity-workbench-panel";
import type {
  EntityResponse,
  FieldResponse,
  PublishedSchemaResponse,
  RuntimeRecordResponse,
} from "@/lib/api";

type EntityWorkbenchPanelProps = {
  entityLogicalName: string;
  initialEntities: EntityResponse[];
  initialFields: FieldResponse[];
  initialPublishedSchema: PublishedSchemaResponse | null;
  initialRecords: RuntimeRecordResponse[];
};

export function EntityWorkbenchPanel({
  entityLogicalName,
  initialEntities,
  initialFields,
  initialPublishedSchema,
  initialRecords,
}: EntityWorkbenchPanelProps) {
  const panel = useEntityWorkbenchPanel({
    entityLogicalName,
    initialEntities,
    initialFields,
    initialPublishedSchema,
    initialRecords,
  });

  return (
    <div className="space-y-8">
      <WorkbenchOverview
        activeSection={panel.activeSection}
        fieldCount={panel.initialFields.length}
        hasPublishedSchema={panel.initialPublishedSchema !== null}
        onSectionChange={panel.setActiveSection}
        publishedVersion={panel.initialPublishedSchema?.version ?? null}
        recordCount={panel.displayedRecords.length}
      />

      {panel.activeSection === "schema" ? (
        <SchemaDesignSection
          calculationExpressionText={panel.calculationExpressionText}
          defaultValueText={panel.defaultValueText}
          displayName={panel.displayName}
          entities={panel.initialEntities}
          fieldType={panel.fieldType}
          handlePublish={panel.handlePublish}
          handlePublishChecks={panel.handlePublishChecks}
          handleSaveField={panel.handleSaveField}
          initialFields={panel.initialFields}
          initialPublishedSchema={panel.initialPublishedSchema}
          isCheckingPublish={panel.isCheckingPublish}
          isPublishing={panel.isPublishing}
          isRequired={panel.isRequired}
          isSavingField={panel.isSavingField}
          isUnique={panel.isUnique}
          logicalName={panel.logicalName}
          relationAuthoringMode={panel.relationAuthoringMode}
          publishCheckErrors={panel.publishCheckErrors}
          relationTargetEntity={panel.relationTargetEntity}
          secondaryDisplayName={panel.secondaryDisplayName}
          secondaryLogicalName={panel.secondaryLogicalName}
          secondaryRelationTargetEntity={panel.secondaryRelationTargetEntity}
          setRelationAuthoringMode={panel.setRelationAuthoringMode}
          setDefaultValueText={panel.setDefaultValueText}
          setCalculationExpressionText={panel.setCalculationExpressionText}
          setDisplayName={panel.setDisplayName}
          setFieldType={panel.setFieldType}
          setIsRequired={panel.setIsRequired}
          setIsUnique={panel.setIsUnique}
          setLogicalName={panel.setLogicalName}
          setRelationTargetEntity={panel.setRelationTargetEntity}
          setSecondaryDisplayName={panel.setSecondaryDisplayName}
          setSecondaryLogicalName={panel.setSecondaryLogicalName}
          setSecondaryRelationTargetEntity={panel.setSecondaryRelationTargetEntity}
        />
      ) : null}

      {panel.activeSection === "runtime" ? (
        <RuntimeOperationsSection
          activeRuntimeSection={panel.activeRuntimeSection}
          deletingRecordId={panel.deletingRecordId}
          displayedRecords={panel.displayedRecords}
          handleClearQuery={panel.handleClearQuery}
          handleCreateRecord={panel.handleCreateRecord}
          handleDeleteRecord={panel.handleDeleteRecord}
          handleDeleteSelectedPreset={panel.handleDeleteSelectedPreset}
          handleExportQueryPresets={panel.handleExportQueryPresets}
          handleImportQueryPresets={panel.handleImportQueryPresets}
          handleLoadSelectedPreset={panel.handleLoadSelectedPreset}
          handleQueryRecords={panel.handleQueryRecords}
          handleSaveQueryPreset={panel.handleSaveQueryPreset}
          initialPublishedSchema={panel.initialPublishedSchema}
          isCreatingRecord={panel.isCreatingRecord}
          isPresetCopied={panel.isPresetCopied}
          isQueryingRecords={panel.isQueryingRecords}
          isSavingPreset={panel.isSavingPreset}
          presetTransferText={panel.presetTransferText}
          queriedRecords={panel.queriedRecords}
          queryConditionsText={panel.queryConditionsText}
          queryFiltersText={panel.queryFiltersText}
          queryLimitText={panel.queryLimitText}
          queryLogicalMode={panel.queryLogicalMode}
          queryOffsetText={panel.queryOffsetText}
          queryPresetName={panel.queryPresetName}
          queryPresets={panel.queryPresets}
          querySortText={panel.querySortText}
          recordPayload={panel.recordPayload}
          selectedPresetName={panel.selectedPresetName}
          setActiveRuntimeSection={panel.setActiveRuntimeSection}
          setPresetTransferText={panel.setPresetTransferText}
          setQueryConditionsText={panel.setQueryConditionsText}
          setQueryFiltersText={panel.setQueryFiltersText}
          setQueryLimitText={panel.setQueryLimitText}
          setQueryLogicalMode={panel.setQueryLogicalMode}
          setQueryOffsetText={panel.setQueryOffsetText}
          setQueryPresetName={panel.setQueryPresetName}
          setQuerySortText={panel.setQuerySortText}
          setRecordPayload={panel.setRecordPayload}
          setSelectedPresetName={panel.setSelectedPresetName}
        />
      ) : null}

      {panel.errorMessage ? <Notice tone="error">{panel.errorMessage}</Notice> : null}
      {panel.statusMessage ? (
        <Notice tone="success">{panel.statusMessage}</Notice>
      ) : null}
    </div>
  );
}
