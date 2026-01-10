import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Skill } from '@/types'
import { tauri } from '@/lib/tauri'

const SKILLS_KEY = ['skills']

export function useSkills() {
  const queryClient = useQueryClient()

  const skillsQuery = useQuery({
    queryKey: SKILLS_KEY,
    queryFn: tauri.getSkills,
  })

  const createSkill = useMutation({
    mutationFn: (data: Partial<Skill>) => tauri.createSkill(data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SKILLS_KEY }),
  })

  const updateSkill = useMutation({
    mutationFn: ({ id, data }: { id: number; data: Partial<Skill> }) =>
      tauri.updateSkill(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SKILLS_KEY }),
  })

  const deleteSkill = useMutation({
    mutationFn: (id: number) => tauri.deleteSkill(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SKILLS_KEY }),
  })

  return { skillsQuery, createSkill, updateSkill, deleteSkill }
}
