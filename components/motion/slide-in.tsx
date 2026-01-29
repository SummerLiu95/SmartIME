"use client"

import { motion, HTMLMotionProps } from "framer-motion"
import { cn } from "@/lib/utils"

interface SlideInProps extends HTMLMotionProps<"div"> {
  direction?: "up" | "down" | "left" | "right"
  delay?: number
  duration?: number
  offset?: number
}

export function SlideIn({ 
  children, 
  className, 
  direction = "up", 
  delay = 0, 
  duration = 0.5, 
  offset = 20,
  ...props 
}: SlideInProps) {
  const variants = {
    hidden: { 
      opacity: 0, 
      x: direction === "left" ? offset : direction === "right" ? -offset : 0,
      y: direction === "up" ? offset : direction === "down" ? -offset : 0
    },
    visible: { 
      opacity: 1, 
      x: 0, 
      y: 0 
    }
  }

  return (
    <motion.div
      initial="hidden"
      animate="visible"
      exit="hidden"
      variants={variants}
      transition={{ 
        duration, 
        delay, 
        ease: [0.21, 0.47, 0.32, 0.98] 
      }}
      className={cn(className)}
      {...props}
    >
      {children}
    </motion.div>
  )
}
