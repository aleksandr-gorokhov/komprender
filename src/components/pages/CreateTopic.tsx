'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form.tsx';
import { Button } from '@/components/ui/button.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select.tsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useNavigate } from 'react-router-dom';

const formSchema = z.object({
  name: z.string().min(2, {
    message: 'Topic name must be at least 2 characters.',
  }),
  partitions: z.coerce.number({ invalid_type_error: 'Number of partitions must be a number.' }).min(1, {
    message: 'Number of partitions must be at least 1.',
  }),
  cleanupPolicy: z.enum(['delete', 'compact', 'compact,delete']),
  insyncReplicas: z.coerce.number({ invalid_type_error: 'Min insync replicas must be a number.' }).min(1, {
    message: 'Min insync replicas must be at least 1.',
  }),
  replicationFactor: z.coerce.number({ invalid_type_error: 'Replication factor must be a number.' }).min(1, {
    message: 'Replication factor must be at least 1.',
  }),
  retentionTime: z.coerce.number({ invalid_type_error: 'Time to retain data must be a number.' }).min(1, {
    message: 'Time to retain data must be at least 1 ms.',
  }),
  sizeLimit: z.coerce.number().nonnegative().optional(),
  maximumMessageSize: z.coerce.number().positive().optional(),
});

export function CreateTopic() {
  const navigate = useNavigate();
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    mode: 'onSubmit',
    defaultValues: {
      name: '',
      partitions: 1,
      cleanupPolicy: 'delete',
      insyncReplicas: 1,
      replicationFactor: 1,
      retentionTime: 604800000,
      sizeLimit: 0,
    },
  });

  async function onSubmit(values: z.infer<typeof formSchema>) {
    try {
      await invoke('create_topic', {
        topic: {
          cleanup_policy: values.cleanupPolicy,
          insync_replicas: values.insyncReplicas,
          replication_factor: values.replicationFactor,
          retention_time: values.retentionTime,
          size_limit: values.sizeLimit,
          ...values,
        },
      });
      navigate('/topics');
    } catch (err) {
      console.error(err);
    }
  }

  function preventString(e: any) {
    e.target.value = e.target.value.replace(/\D/g, '');
  }

  return (
    <div className="flex p-6 w-1/2">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8 w-full">
          <FormField
            control={form.control}
            name="name"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Topic Name</FormLabel>
                <FormControl>
                  <Input placeholder="" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <div className="flex space-x-4">
            <FormField
              control={form.control}
              name="partitions"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>Number of partitions</FormLabel>
                  <FormControl>
                    <Input placeholder="" onInput={e => preventString(e)} {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="cleanupPolicy"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>CleanUp Policy</FormLabel>
                  <Select onValueChange={field.onChange} defaultValue={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="delete">Delete</SelectItem>
                      <SelectItem value="compact">Compact</SelectItem>
                      <SelectItem value="compact,delete">Compact,Delete</SelectItem>
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
          <div className="flex space-x-4">
            <FormField
              control={form.control}
              name="insyncReplicas"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>Min insync replicas</FormLabel>
                  <FormControl>
                    <Input placeholder="" onInput={e => preventString(e)} {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="replicationFactor"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>Replication factor</FormLabel>
                  <FormControl>
                    <Input placeholder="" onInput={e => preventString(e)} {...field} />
                  </FormControl>
                  <FormMessage />
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
          <FormField
            control={form.control}
            name="retentionTime"
            render={({ field }) => (
              <FormItem className="w-full">
                <FormLabel>Time to retain data (in ms) 7 days default</FormLabel>
                <FormControl>
                  <Input placeholder="" onInput={e => preventString(e)} {...field} />
                </FormControl>
                <FormMessage />
                <FormMessage />
              </FormItem>
            )}
          />
          <div className="flex space-x-4">
            <FormField
              control={form.control}
              name="maximumMessageSize"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>Maximum message size in bytes</FormLabel>
                  <FormControl>
                    <Input placeholder="" onInput={e => preventString(e)} {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="sizeLimit"
              render={({ field }) => (
                <FormItem className="w-1/2 max-w-1/2">
                  <FormLabel>Max size on disk</FormLabel>
                  <Select onValueChange={field.onChange} defaultValue="0">
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      <SelectItem value="0">Not Set</SelectItem>
                      <SelectItem value="1">1 GB</SelectItem>
                      <SelectItem value="10">10 GB</SelectItem>
                      <SelectItem value="20">20 GB</SelectItem>
                      <SelectItem value="50">50 GB</SelectItem>
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
          <Button type="submit" className="w-full">
            Create
          </Button>
        </form>
      </Form>
    </div>
  );
}
